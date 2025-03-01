use super::Resource;
use ash::vk;

enum BufferCopyDestination {
    BUFFER(vk::Buffer),
    IMAGE(vk::Image)
}

pub struct BufferCopyInfo {
    buff: vk::Buffer,
    size: usize,
    dst: BufferCopyDestination,

    signal_semaphores: Vec<vk::Semaphore>
}

pub struct ResourceQueue {
    copy_infos: Vec<BufferCopyInfo>
}

impl ResourceQueue {
    pub fn new() -> ResourceQueue {
        ResourceQueue { copy_infos: Vec::new() }
    }

    pub fn add_copy_op(&mut self, copy_info: BufferCopyInfo) {
        self.copy_infos.push(copy_info);
    }

    pub fn drain_copy_ops(&mut self) -> Vec<BufferCopyInfo> {
        self.copy_infos.drain(0..self.copy_infos.len()).into_iter().collect()
    } 
}