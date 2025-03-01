use ash::vk;
use std::{cell::RefCell, pin::Pin};

#[derive(Clone, Copy)]
pub enum QueueType {
    GRAPHICS,
    TRANSFER,
    BOTH
}

pub struct QueueFamily {
    queue_type: QueueType,
    queues: Vec<vk::Queue>,
    index: u32,
    
    current_queue_index: RefCell<usize>
}

impl QueueFamily {
    pub fn new(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Vec<QueueFamily> {
        let mut queue_families = Vec::<QueueFamily>::new();

        let properties = unsafe{
            instance.get_physical_device_queue_family_properties(physical_device)
        };

        for property in properties.iter().enumerate() {
            let supports_graphics = property.1.queue_flags & vk::QueueFlags::GRAPHICS == vk::QueueFlags::GRAPHICS;
            let supports_transfer = property.1.queue_flags & vk::QueueFlags::TRANSFER == vk::QueueFlags::TRANSFER;


            if !supports_graphics && !supports_transfer {
                continue;
            }

            let queue_type = if supports_graphics && supports_transfer {
                QueueType::BOTH
            } else if supports_graphics{
                QueueType::GRAPHICS
            } else  {
                QueueType::TRANSFER
            };

            queue_families.push(
                QueueFamily { queue_type: queue_type, queues: Vec::with_capacity(property.1.queue_count as usize), index: property.0 as u32, current_queue_index: RefCell::new(0)}
            );

        }

        queue_families
    }

    pub fn get_queue_create_info(&self) -> (vk::DeviceQueueCreateInfo, Pin<Vec<f32>>){
        let priorities = Pin::new(vec![1.0f32; self.queues.capacity()]);

        (vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            queue_family_index: self.index,
            queue_count: self.queues.capacity() as u32,
            p_queue_priorities: priorities.as_ptr(),
            ..Default::default()
        },
        priorities)
    }

    pub fn fill_with_queues(&mut self, device: &ash::Device) {
        assert!(self.queues.len() < self.queues.capacity() - 1);

        for i in 0..self.queues.capacity() {
            self.queues.push(
                unsafe { device.get_device_queue(self.index, i as u32) }
            );
        }
    }

    pub fn get_queue(&self) -> vk::Queue{
        let current_index = *self.current_queue_index.borrow();
        let queue = self.queues[current_index];

        *self.current_queue_index.borrow_mut() = (current_index + 1) % self.queues.len();

        queue
    }

    pub fn get_type(&self) -> QueueType {
        self.queue_type
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }
}