use std::ffi::c_void;

use ash::vk;
use glfw::init;

use crate::engine::Device;

pub struct TimelineSemaphore<'a> {
    semaphore: vk::Semaphore,

    device: &'a Device
}

impl<'a> TimelineSemaphore<'a> {
    pub fn new(device: &'a Device, initial_value: u64) -> TimelineSemaphore<'a> {
        let semaphore = TimelineSemaphore::create_semaphore(device, initial_value);

        TimelineSemaphore { semaphore, device }
    }

    fn create_semaphore(device: &Device, initial_value: u64) -> vk::Semaphore{
        let timeline_info = vk::SemaphoreTypeCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_TYPE_CREATE_INFO,
            semaphore_type: vk::SemaphoreType::TIMELINE,
            initial_value,

            ..Default::default()
        };

        let create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,

            p_next: &timeline_info as *const _ as *const c_void,

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_semaphore(&create_info, None)
        }.expect("Failed to create a semaphore")
    }

    pub fn get_semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }

    pub fn get_value(&self) -> u64 {
        unsafe {
            self.device.get_ash_device().get_semaphore_counter_value(self.semaphore)
        }.expect("Failed to get the timeline semaphore value")
    }
}