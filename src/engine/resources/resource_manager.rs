use crate::engine::{CommandBuffer, CommandPool, Device, Semaphore};
use ash::vk;

use super::*;

const STANDBY_PROCESS_COUNT: usize = 3;

#[derive(Clone)]
struct Process<'a> {
    pub command_buffer: CommandBuffer<'a>,
    pub semaphore: Semaphore<'a>,
    pub finished_value: u64,
    pub busy: bool
}

pub struct ResourceManager<'a> {
    device: &'a Device,

    command_pool: CommandPool<'a>,
    processes: Vec<Process<'a>>
}

impl<'a> ResourceManager<'a> {
    pub fn new(device: &'a Device) -> ResourceManager<'a>{
        let command_pool: CommandPool<'a> = CommandPool::new(device, crate::engine::QueueType::TRANSFER, false, true);
        let processes: Vec<Process<'a>> = ResourceManager::create_processes(device, &command_pool, STANDBY_PROCESS_COUNT);

        ResourceManager { device, processes, command_pool }
    } 

    fn create_processes(device: &'a Device, command_pool: &CommandPool<'a>, process_count: usize) -> Vec<Process<'a>> {
        let command_buffers: Vec<CommandBuffer<'a>> = command_pool.allocate_command_buffers(false, 3);

        let mut processes: Vec<Process<'a>> = Vec::with_capacity(process_count);

        for command_buffer in command_buffers {
            processes.push(Process {command_buffer,
                                    semaphore: Semaphore::new(device, crate::engine::SemaphoreType::TIMELINE, 0),
                                    finished_value: 1,
                                    busy: false });
        }

        processes
    }

    fn get_free_process(&mut self) -> Process<'a> {
        let mut process: Option<Process<'a>> = None;

        for curr_process in self.processes.iter() {
            if curr_process.busy {
                continue;
            }

            process = Some(curr_process.clone());
        }

        if matches!(process, None) {
            process = Some(ResourceManager::create_processes(&self.device, &self.command_pool, 1).remove(0));
        }

        process.as_mut().unwrap().busy = true;

        process.unwrap()
    }

    fn submit_queue(&mut self, queue: &mut ResourceQueue) {
        let copy_ops = queue.drain_copy_ops();
        let process = self.get_free_process();

        process.command_buffer.begin(None);

        unsafe{
            for op in copy_ops.iter() {
                match op.dst {
                    resource_queue::BufferCopyDestination::BUFFER(buff) => {
                        let region = vk::BufferCopy {
                            size: op.size as u64,
                            src_offset: 0,
                            dst_offset: 0
                        };

                        self.device.get_ash_device().cmd_copy_buffer(process.command_buffer.get_command_buffer(), op.buff, buff, &[region]);
                    },
                    resource_queue::BufferCopyDestination::IMAGE(img) => {
                        let region = vk::BufferCopy {
                            size: op.size as u64,
                            src_offset: 0,
                            dst_offset: 0
                        };

//                        self.device.get_ash_device().cmd_copy_buffer_to_image(process.command_buffer.get_command_buffer(), op.buff, img, &[region]);
                        todo!("layout bs");
                    }
                }
            }
        }
    
        process.command_buffer.end();
    }
}