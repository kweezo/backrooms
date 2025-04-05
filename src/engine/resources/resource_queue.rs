use ash::vk;

use super::ImageCopyInfo;

pub enum BufferCopyDestination {
    BUFFER(vk::Buffer),
    IMAGE(ImageCopyInfo)
}

pub struct BufferCopyInfo {
    pub buff: vk::Buffer,
    pub size: usize,
    pub dst: BufferCopyDestination,

    pub signal_semaphores: Vec<vk::Semaphore>
}

pub struct ResourceQueue {
    copy_infos: Vec<BufferCopyInfo>
}

impl ResourceQueue {
    pub fn new() -> ResourceQueue {
        ResourceQueue { copy_infos: Vec::new() }
    }

    pub fn add_copy_ops(&mut self, copy_infos: Vec<BufferCopyInfo>) {
        self.copy_infos.extend(copy_infos);
    }

    pub fn drain_copy_ops(&mut self) -> Vec<BufferCopyInfo> {
        assert!(!self.copy_infos.is_empty());

        self.copy_infos.drain(0..self.copy_infos.len()).into_iter().collect()
    } 
}