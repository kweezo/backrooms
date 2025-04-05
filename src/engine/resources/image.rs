use ash::vk;
use vk_mem::Alloc;

use crate::engine::{resources::resource_queue::BufferCopyDestination, Device, QueueType};

use super::{buffer, Buffer, BufferCopyInfo};

#[derive(Clone, Copy)]
pub enum ImageType {
    SAMPLED,
    STORAGE,
    COLOR,
}

pub enum ImageLayout {
    UNDEFINED,
    GENERAL,
    COLOR
}

pub enum ImageAspect {
    NONE,
    COLOR,
    DEPTH
}

struct RawImg {
    image: vk::Image,
    allocation: vk_mem::Allocation,
}

pub struct ImageCopyInfo {
    pub image: vk::Image,
    pub layout: vk::ImageLayout,
    pub aspect: vk::ImageAspectFlags
}

pub struct ImageCreateInfo {
    pub width: u32,
    pub height: u32,
    pub image_type: ImageType,
    pub image_layout: ImageLayout,
    pub format: vk::Format,
    pub aspect: ImageAspect
}

pub struct Image<'a> {
    device: &'a Device,

    raw_image: RawImg,
    ptr: Option<*mut u8>,

    staging_buf: Option<buffer::RawBuf>,

    dimensions: vk::Extent2D,
    size: usize,

    updated_frequently: bool,
    on_device_memory: bool,

    final_layout: vk::ImageLayout,
    aspect: vk::ImageAspectFlags,

    signal_semaphores: Vec<vk::Semaphore>
}

impl<'a> Image<'a> {
    pub fn new(device: &'a Device, data: &[u8], info: &ImageCreateInfo, updated_frequently: bool, on_device_memory: bool) -> Image<'a> {
        let (mut raw_image, ptr)= Image::create_image(device, vk::Extent2D { width: info.width, height: info.height}, data.len(), on_device_memory,
             updated_frequently, info.image_type, info.format);

        let final_layout = match info.image_layout {
                ImageLayout::COLOR => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ImageLayout::GENERAL => vk::ImageLayout::GENERAL,
                ImageLayout::UNDEFINED => vk::ImageLayout::UNDEFINED
        };

        let aspect = match info.aspect {
            ImageAspect::NONE => vk::ImageAspectFlags::NONE,
            ImageAspect::COLOR => vk::ImageAspectFlags::COLOR,
            ImageAspect::DEPTH => vk::ImageAspectFlags::DEPTH
        };

        if !on_device_memory {
            if updated_frequently {
                Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);

                return Image { device, raw_image, staging_buf: None, ptr: Some(ptr.unwrap()), dimensions: vk::Extent2D { width: info.width, height: info.height},
                 size: data.len(), on_device_memory, updated_frequently, final_layout, aspect, signal_semaphores: Vec::new() };
            } 

            Buffer::copy_data_to_buffer(device, &mut raw_image.allocation, data);

            return Image { device, raw_image, staging_buf: None, ptr: None, dimensions: vk::Extent2D { width: info.width, height: info.height },
             size: data.len(), on_device_memory, updated_frequently, final_layout, aspect, signal_semaphores: Vec::new() }
        }

        let (mut raw_staging_buf, ptr) = Buffer::create_staging_buffer(device, data.len(), updated_frequently);

        if updated_frequently {
            Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);
        } else {
            Buffer::copy_data_to_buffer(device, &mut raw_staging_buf.allocation, data);
        }

        Image { device, raw_image, ptr, staging_buf: Some(raw_staging_buf), dimensions: vk::Extent2D { width: info.width, height: info.height }, size: data.len(),
         updated_frequently: updated_frequently, on_device_memory: on_device_memory, final_layout, aspect, signal_semaphores: Vec::new() }
    }

    fn create_image(device: &Device, dimensions: vk::Extent2D, size: usize, on_device_memory: bool, updated_frequently: bool, image_type: ImageType, format: vk::Format) -> (RawImg, Option<*mut u8>){
        let queue_family_indices =
         device.get_queue_family_indices(vec![QueueType::TRANSFER, QueueType::GRAPHICS]);

        let sharing_mode = if queue_family_indices.len() > 1 {
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let mut image_flags = match image_type {
            ImageType::STORAGE => vk::ImageUsageFlags::STORAGE,
            ImageType::COLOR => vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ImageType::SAMPLED => vk::ImageUsageFlags::SAMPLED 
        };

        if on_device_memory {
            image_flags |= vk::ImageUsageFlags::TRANSFER_DST;
        }

        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,

            image_type: vk::ImageType::TYPE_2D,

            format,
            extent: vk::Extent3D{width: dimensions.width, height: dimensions.height, depth: 1 },

            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,

            sharing_mode,

            queue_family_index_count: queue_family_indices.len() as u32,
            p_queue_family_indices: queue_family_indices.as_ptr(),

            initial_layout: vk::ImageLayout::UNDEFINED,

            usage: image_flags,

            ..Default::default()
        };


        let alloc_usage; 
        let flags;
        if on_device_memory {
            alloc_usage = vk_mem::MemoryUsage::AutoPreferDevice;
            flags = vk_mem::AllocationCreateFlags::empty();
        } else {
            todo!("On host images are as of yet not supported");
        };

        
        let allocation_info = vk_mem::AllocationCreateInfo {
            flags,

            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            usage: alloc_usage,

            user_data: size,

            ..Default::default()
        };


        let (image, mut allocation) = unsafe {
            device.get_allocator()
            .create_image(&image_info, &allocation_info)
        }.expect("Failed to allocate a buffer");

        if updated_frequently && !on_device_memory{
            unsafe {
                let ptr = device.get_allocator().map_memory(&mut allocation).expect("Failed to map persistent staging memory");

                return (RawImg {image, allocation}, Some(ptr));
            }
        }

        (RawImg {image, allocation }, None)
    }

    pub fn set_signal_semaphores(&mut self, signal_semaphores: &Vec<vk::Semaphore>) {
        self.signal_semaphores = signal_semaphores.clone();
    }

    pub fn get_copy_op(&self) -> BufferCopyInfo {
        assert!(self.on_device_memory, "Tried to get copy op info for an on host buffer");

        let image_copy_info = ImageCopyInfo {
            image: self.raw_image.image,
            layout: self.final_layout,
            aspect: self.aspect
        };

        BufferCopyInfo {
            buff: self.staging_buf.as_ref().unwrap().buffer,
            dst: BufferCopyDestination::IMAGE(image_copy_info),

            size: self.size,

            signal_semaphores: self.signal_semaphores.clone()
        }
    }
}

impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe {
            if self.updated_frequently {
                if self.on_device_memory {
                    self.device.get_allocator().unmap_memory(&mut self.staging_buf.as_mut().unwrap().allocation);
                } else {
                    self.device.get_allocator().unmap_memory(&mut self.raw_image.allocation);
                }
            }

            if self.on_device_memory {
                self.device.get_allocator().destroy_buffer(self.staging_buf.as_ref().unwrap().buffer, &mut self.staging_buf.as_mut().unwrap().allocation);
            }

            self.device.get_allocator().destroy_image(self.raw_image.image, &mut self.raw_image.allocation);
        }
    }
}