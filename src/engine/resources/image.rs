use ash::vk;

use crate::engine::{Device, QueueType};

use super::{buffer, Buffer};

pub enum ImageType {
    SAMPLED,
    STORAGE,
    COLOR,
}

struct RawImg {
    image: vk::Image,
    allocation: vk_mem::Allocation
}

pub struct Image<'a> {
    device: &'a Device,

    staging_buf: Option<buffer::RawBuf>,

    size: vk::Extent2D,

    updated_frequently: bool,
    on_device_memory: bool,
}

impl<'a> Image<'a> {
    pub fn new(device: &'a Device, data: &[u8], width: u32, height: u32, updated_frequently: bool, on_device_memory: bool, image_type: ImageType) {
        let (mut raw_buf, ptr)= Buffer::create_buffer(device, data.len(), buffer_type, on_device_memory, updated_frequently);

        if !on_device_memory {
            if updated_frequently {
                Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);

                return Buffer { device, raw_buf, raw_staging_buf: None, ptr: Some(ptr.unwrap()), size: data.len(), on_device_memory, updated_frequently, signal_semaphores: Vec::new() }
            } 

            Buffer::copy_data_to_buffer(device, &mut raw_buf, data);

            return Buffer { device, raw_buf, raw_staging_buf: None, ptr: None, size: data.len(), on_device_memory, updated_frequently, signal_semaphores: Vec::new() };
        }

        let (mut raw_staging_buf, ptr) = Buffer::create_staging_buffer(device, data.len(), updated_frequently);

        if updated_frequently {
            Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);
        } else {
            Buffer::copy_data_to_buffer(device, &mut raw_staging_buf, data);
        }
    }

    fn create_image(device: &Device, size: usize, on_device_memory: bool, updated_frequently: bool, image_type: ImageType) {
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
            format: vk::Format::
        }

        
    }
}