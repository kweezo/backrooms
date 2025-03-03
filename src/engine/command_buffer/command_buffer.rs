use crate::engine::{Device, Fence, QueueType, Semaphore};
use ash::vk;

pub struct CommandBufferSubmitInfo<'a> {
    pub command_buffer: vk::CommandBuffer,
    pub wait_semaphores: Vec<(Semaphore<'a>, u64)>,
    pub stage_flags: Vec<vk::PipelineStageFlags2>,
    pub signal_semaphores: Vec<(Semaphore<'a>, u64)>
}

pub struct RenderPassInheritanceInfo {
    pub render_pass: vk::RenderPass,

    pub subpass: u32,
    framebuffer: vk::Framebuffer
}

#[derive(Clone)]
pub struct CommandBuffer<'a> {
    device: &'a Device,

    command_buffer: vk::CommandBuffer,
    secondary: bool,

    wait_semaphores: Vec<(Semaphore<'a>, u64)>,
    stage_flags: Vec<vk::PipelineStageFlags2>,

    signal_semaphores: Vec<(Semaphore<'a>, u64)>
}

impl<'a> CommandBuffer<'a> {
    pub fn new(device: &'a Device, command_buffer: vk::CommandBuffer, secondary: bool) -> CommandBuffer<'a>{
        CommandBuffer { device, command_buffer, secondary, wait_semaphores: Vec::new(), signal_semaphores: Vec::new(), stage_flags: Vec::new() }
    }

    pub fn get_inheritance_info(&self, render_pass_inheritance_info: Option<RenderPassInheritanceInfo>) -> vk::CommandBufferInheritanceInfo{
        assert!(!self.secondary,"Tried to get inheritance info for a primary command buffer");

        match render_pass_inheritance_info {
            None => {
                vk::CommandBufferInheritanceInfo { 
                    s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_INFO,

                    ..Default::default()
                }
            },

            Some(info) => {
                vk::CommandBufferInheritanceInfo {
                    s_type: vk::StructureType::COMMAND_BUFFER_INHERITANCE_INFO,

                    render_pass: info.render_pass,
                    framebuffer: info.framebuffer,
                    subpass: info.subpass,

                    ..Default::default()
                }
            }
        }
    }

    pub fn begin(&self, render_pass_inheritance_info: Option<RenderPassInheritanceInfo>) {
        let inheritance_info = if self.secondary {
            self.get_inheritance_info(render_pass_inheritance_info)
        } else {
            vk::CommandBufferInheritanceInfo::default()
        };

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,

            p_inheritance_info: &inheritance_info,
            
            ..Default::default()
        };

        unsafe {
            self.device.get_ash_device().begin_command_buffer(self.command_buffer, &begin_info)
             .expect("Failed to begin the command buffer");
        }
    }

    pub fn end(&self) {
        unsafe {
            self.device.get_ash_device().end_command_buffer(self.command_buffer)
             .expect("Failed to end the command buffer");
        }
    }

    pub fn add_signal_semaphores(&mut self, semaphores: Vec<(Semaphore<'a>, u64)>) {

        self.signal_semaphores.extend(semaphores);
    }

    pub fn add_wait_semaphores(&mut self, semaphores: Vec<(Semaphore<'a>, vk::PipelineStageFlags2, u64)>) {
        let semaphore_handles: Vec<(Semaphore, u64)> = 
         semaphores
         .iter()
         .map(|s| (s.0, s.2))
         .collect();

        let stage_flags: Vec<vk::PipelineStageFlags2> = 
         semaphores
         .iter()
         .map(|s| s.1)
         .collect();



        self.wait_semaphores.extend(semaphore_handles);
        self.stage_flags.extend(stage_flags);
    }

    pub fn get_submit_info(&mut self, clear_semaphores: bool) -> CommandBufferSubmitInfo {
        //TODO is clear semaphores useful?
        if clear_semaphores {
            return CommandBufferSubmitInfo{
                command_buffer: self.command_buffer,
                wait_semaphores: self.wait_semaphores.drain(0..self.wait_semaphores.len()).collect(),
                signal_semaphores: self.signal_semaphores.drain(0..self.signal_semaphores.len()).collect(),
                stage_flags: self.stage_flags.clone()
            };
        }
        
        return CommandBufferSubmitInfo{
            command_buffer: self.command_buffer,
            wait_semaphores: self.wait_semaphores.clone(),
            signal_semaphores: self.signal_semaphores.clone(),
            stage_flags: self.stage_flags.clone()
        };
    }
    pub fn submit_buffers(device: &Device, fence: Option<Fence>, queue_type: QueueType, submit_infos: &Vec<CommandBufferSubmitInfo>) {
        let mut command_buffers = Vec::with_capacity(submit_infos.len());
        let mut wait_semaphores = Vec::with_capacity(submit_infos.len());
        let mut signal_semaphores = Vec::with_capacity(submit_infos.len());
        let mut stage_flags = Vec::with_capacity(submit_infos.len());

        for submit_info in submit_infos.iter() {
            stage_flags.extend(submit_info.stage_flags.clone());

            for (i, semaphore) in submit_info.wait_semaphores.iter().enumerate() {
                wait_semaphores.push(vk::SemaphoreSubmitInfo {
                    s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,

                    semaphore: semaphore.0.get_semaphore(),
                    value: semaphore.1,
                    stage_mask: submit_info.stage_flags[i], 

                    ..Default::default() 
                });
            }

            for semaphore in submit_info.signal_semaphores.iter() {
                signal_semaphores.push(vk::SemaphoreSubmitInfo {
                    s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,

                    semaphore: semaphore.0.get_semaphore(),
                    value: semaphore.1,

                    ..Default::default() 
                });
            }

            command_buffers.push(vk::CommandBufferSubmitInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_SUBMIT_INFO,

                command_buffer: submit_info.command_buffer,

                ..Default::default() 
            });
        }

        let vk_submit_info = vk::SubmitInfo2 {
            s_type: vk::StructureType::SUBMIT_INFO,

            command_buffer_info_count: command_buffers.len() as u32,
            p_command_buffer_infos: command_buffers.as_ptr(),

            wait_semaphore_info_count: wait_semaphores.len() as u32,
            p_wait_semaphore_infos: wait_semaphores.as_ptr(),

            signal_semaphore_info_count: signal_semaphores.len() as u32,
            p_signal_semaphore_infos: signal_semaphores.as_ptr(),

            ..Default::default()
        };

        let fence_raw = match fence {
            None => vk::Fence::null(),
            Some(handle) => handle.get_fence()
        };

        unsafe {
            device.get_ash_device().queue_submit2(device.get_queue(queue_type), &[vk_submit_info], fence_raw)
        }.expect("Couldn't submit command buffers");

    }

    pub fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}