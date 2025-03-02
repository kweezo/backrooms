use crate::engine::{CommandBuffer, CommandPool, Device};
use ash::vk;

use super::*;

const STANDBY_COMMAND_BUFFER_COUNT: usize = 3;

pub struct ResourceManager<'a> {
    device: &'a Device,

    command_pool: CommandPool<'a>,
    command_buffers: Vec<CommandBuffer<'a>>
}

impl<'a> ResourceManager<'a> {
    pub fn new(device: &'a Device) -> ResourceManager<'a>{
        let command_pool = CommandPool::new(device, crate::engine::QueueType::TRANSFER, false, true);
        let command_buffers = command_pool.allocate_command_buffers(false, 3);

        ResourceManager { device , command_buffers, command_pool }
    } 
}