use crate::engine::{Device, QueueFamily, QueueType};
use ash::vk::{self, Queue};
use vk_mem::Alloc;


pub enum BufferType {
    Vertex,
    Index,
    Uniform,
    Storage
}

struct RawBuf {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
}

pub struct Buffer<'a> {
    device: &'a Device,

    raw_buf: RawBuf,
    raw_staging_buf: Option<RawBuf>,
    ptr: Option<*mut u8>,

    size: usize,

    on_device_memory: bool,
    updated_frequently: bool,
}

impl<'a> Buffer<'a> {
    pub fn new(
        device: &'a Device,

        on_device_memory: bool,
        updated_frequently: bool,

        buffer_type: BufferType,

        data: &[u8]        
    ) -> Buffer<'a>{
        let (mut raw_buf, ptr)= Buffer::create_buffer(device, data.len(), buffer_type, on_device_memory, updated_frequently);

        if !on_device_memory {
            if updated_frequently {
                Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);

                return Buffer { device, raw_buf, raw_staging_buf: None, ptr: Some(ptr.unwrap()), size: data.len(), on_device_memory, updated_frequently };
            } 

            Buffer::copy_data_to_buffer(device, &mut raw_buf, data);

            return Buffer { device, raw_buf, raw_staging_buf: None, ptr: None, size: data.len(), on_device_memory, updated_frequently };
        }

        let (mut raw_staging_buf, ptr) = Buffer::create_staging_buffer(device, data.len(), updated_frequently);

        if updated_frequently {
            Buffer::copy_data_to_persistent_buffer(ptr.unwrap(), data);
        } else {
            Buffer::copy_data_to_buffer(device, &mut raw_staging_buf, data);
        }

        Buffer { device, raw_buf, raw_staging_buf: Some(raw_staging_buf), ptr, size: data.len(), on_device_memory, updated_frequently }
    }

    fn create_buffer(device: &Device, size: usize, buffer_type: BufferType, on_device_memory: bool, updated_frequently: bool) -> (RawBuf, Option<*mut u8>) {
        let queue_family_indices =
         device.get_queue_family_indices(vec![QueueType::TRANSFER, QueueType::GRAPHICS]);

        let sharing_mode = if queue_family_indices.len() > 1 {
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let mut buff_usage = match buffer_type {
            BufferType::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferType::Index => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
            BufferType::Storage => vk::BufferUsageFlags::STORAGE_BUFFER
        } ;

        if on_device_memory {
            buff_usage |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,

            usage: buff_usage,
            size: size as u64,
        
            sharing_mode,
            queue_family_index_count: queue_family_indices.len() as u32,
            p_queue_family_indices: queue_family_indices.as_ptr(),

            ..Default::default()
        };


        let alloc_usage; 
        let flags;
        if on_device_memory {
            alloc_usage = vk_mem::MemoryUsage::AutoPreferDevice;
            flags = vk_mem::AllocationCreateFlags::empty();
        } else {
            alloc_usage = vk_mem::MemoryUsage::AutoPreferHost;
            flags = vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE;
        };

        
        let allocation_info = vk_mem::AllocationCreateInfo {
            flags,

            required_flags: vk::MemoryPropertyFlags::empty(),
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            usage: alloc_usage,

            user_data: size,

            ..Default::default()
        };
 

        let (buffer, mut allocation) = unsafe {
            device.get_allocator()
            .create_buffer(&buffer_info, &allocation_info)
        }.expect("Failed to allocate a buffer");

        if updated_frequently && !on_device_memory{
            unsafe {
                let ptr = device.get_allocator().map_memory(&mut allocation).expect("Failed to map persistent staging memory");

                return (RawBuf {buffer, allocation}, Some(ptr));
            }
        }

        (RawBuf { buffer, allocation }, None)

    }

    fn create_staging_buffer(device: &Device, size: usize, updated_frequently: bool) -> (RawBuf, Option<*mut u8>) {
        let queue_family_indices =
         device.get_queue_family_indices(vec![QueueType::TRANSFER]);

        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,

            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            size: size as u64,
        
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 1,
            p_queue_family_indices: queue_family_indices.as_ptr(),

            ..Default::default()
        };


        let flags = vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | if updated_frequently {
            vk_mem::AllocationCreateFlags::STRATEGY_BEST_FIT | vk_mem::AllocationCreateFlags::MAPPED
        } else {
            vk_mem::AllocationCreateFlags::STRATEGY_MIN_TIME
        };

        
        let allocation_info = vk_mem::AllocationCreateInfo {
            flags,

            required_flags: vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            preferred_flags: vk::MemoryPropertyFlags::empty(),
            usage: vk_mem::MemoryUsage::AutoPreferHost,

            user_data: size,

            ..Default::default()
        };
 

        let (buffer, mut allocation) = unsafe {
            device.get_allocator()
            .create_buffer(&buffer_info, &allocation_info)
        }.expect("Failed to allocate a buffer");

        if updated_frequently {
            unsafe {
                let ptr = device.get_allocator().map_memory(&mut allocation).expect("Failed to map persistent staging memory");

                return (RawBuf {buffer, allocation}, Some(ptr));
            }
        }

        (RawBuf { buffer, allocation }, None)
    }

    fn copy_data_to_buffer(device: &Device, buf: &mut RawBuf, data: &[u8]) {
        unsafe{
            let dst_ptr = device.get_allocator().map_memory(&mut buf.allocation)
             .expect("Failed to map buffer memory");

            std::ptr::copy(data.as_ptr(), dst_ptr, data.len());
            //Sometimes I like to let jesus take the wheel

            device.get_allocator().unmap_memory(&mut buf.allocation);
        }
    }

    fn copy_data_to_persistent_buffer(ptr: *mut u8, data: &[u8]) {
        unsafe {
            std::ptr::copy(data.as_ptr(), ptr, data.len());
        }
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        unsafe {
            if self.updated_frequently {
                if self.on_device_memory {
                    self.device.get_allocator().unmap_memory(&mut self.raw_staging_buf.as_mut().unwrap().allocation);
                } else {
                    self.device.get_allocator().unmap_memory(&mut self.raw_buf.allocation);
                }
            }

            if self.on_device_memory {
                self.device.get_allocator().destroy_buffer(self.raw_staging_buf.as_ref().unwrap().buffer, &mut self.raw_staging_buf.as_mut().unwrap().allocation);
            }

            self.device.get_allocator().destroy_buffer(self.raw_buf.buffer, &mut self.raw_buf.allocation);
        }
    }
}