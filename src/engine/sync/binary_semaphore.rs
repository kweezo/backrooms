use ash::vk;

use crate::engine::Device;

pub struct BinarySemaphore {
    semaphore: vk::Semaphore
}

impl BinarySemaphore {
    pub fn new(device: &Device) -> BinarySemaphore {
        let semaphore = BinarySemaphore::create_semaphore(device);

        BinarySemaphore { semaphore }
    }

    fn create_semaphore(device: &Device) -> vk::Semaphore{
        let create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_semaphore(&create_info, None)
        }.expect("Failed to create a semaphore")
    }

    pub fn get_semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}