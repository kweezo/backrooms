use std::ffi::c_void;

use ash::vk;

use crate::engine::Device;

#[derive(Clone, Copy)]
pub enum SemaphoreType {
    BINARY,
    TIMELINE
}

#[derive(Clone, Copy)]
pub struct Semaphore<'a> {
    semaphore: vk::Semaphore,
    semaphore_type: SemaphoreType,

    device: &'a Device
}

impl<'a> Semaphore<'a> {
    pub fn new(device: &'a Device, semaphore_type: SemaphoreType, initial_value: u64) -> Semaphore<'a> {
        let semaphore = match semaphore_type {
           SemaphoreType::BINARY => Semaphore::create_binary_semaphore(device) ,
           SemaphoreType::TIMELINE => Semaphore::create_timeline_semaphore(device, initial_value)
        };

        Semaphore { semaphore, semaphore_type, device }
    }

    fn create_binary_semaphore(device: &Device) -> vk::Semaphore {
        let create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_semaphore(&create_info, None)
        }.expect("Failed to create a semaphore")

    }

    fn create_timeline_semaphore(device: &Device, initial_value: u64) -> vk::Semaphore{
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

    pub fn get_type(&self) -> SemaphoreType {
        self.semaphore_type
    }

    pub fn get_value(&self) -> u64 {
        assert!(matches!(self.semaphore_type, SemaphoreType::TIMELINE));

        unsafe {
            self.device.get_ash_device().get_semaphore_counter_value(self.semaphore)
        }.expect("Failed to get the timeline semaphore value")
    }
}