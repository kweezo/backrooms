use std::collections::binary_heap;

use ash::vk;

use crate::engine::{Device, QueueType};

use super::CommandBuffer;

pub struct CommandPool<'a> {
    device: &'a Device,
    command_pool: vk::CommandPool
}

impl<'a> CommandPool<'a> {
    pub fn new(device: &'a Device, queue_type: QueueType, short_lived: bool, individual_reset: bool) -> CommandPool<'a> {
        let command_pool = CommandPool::create_command_pool(device, queue_type, short_lived, individual_reset);

        CommandPool { device, command_pool }
    }

    fn create_command_pool( device: &Device, queue_type: QueueType, short_lived: bool, individual_reset: bool) -> vk::CommandPool{
        let queue_family_index = device.get_queue_family_indices(vec![queue_type])[0];

        let mut flags = vk::CommandPoolCreateFlags::empty();

        if short_lived {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if  individual_reset {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }

        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,

            flags,
            queue_family_index,

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_command_pool(&pool_info, None)
        }.expect("Failed to createa command pool")
    }

    fn allocate_command_buffers(&self, secondary: bool, count: u32) -> Vec<CommandBuffer> {
        let level = if secondary {
            vk::CommandBufferLevel::SECONDARY
        } else {
            vk::CommandBufferLevel::PRIMARY
        };

        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,

            level,
            command_pool: self.command_pool,
            command_buffer_count: count,

            ..Default::default()
        };

        let raw_buffers = unsafe {
            self.device.get_ash_device().allocate_command_buffers(&alloc_info)
             .expect("Failed to allocate command buffers")
        };

        let mut command_buffers = Vec::<CommandBuffer>::with_capacity(raw_buffers.len());

        for raw_buf in raw_buffers.iter() {
            command_buffers.push(
                CommandBuffer::new(*raw_buf, secondary)
            );
        }

        command_buffers
    }
}