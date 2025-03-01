use ash::vk;

use crate::engine::Device;

use super::CommandPool;

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
    secondary: bool
}

impl CommandBuffer {
    pub fn new(command_buffer: vk::CommandBuffer, secondary: bool) -> CommandBuffer{
        CommandBuffer { command_buffer, secondary }
    }

}