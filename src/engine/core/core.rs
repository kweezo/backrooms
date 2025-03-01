use crate::window::Window;

use super::*;

#[macro_export]
macro_rules! instance {
    ($x:expr) => {
        $x.get_instance().get_ash_instance()   
    };
}

#[macro_export]
macro_rules! device {
    ($x:expr) => {
        $x.get_device().get_ash_device()   
    };
}

pub struct Core {
    entry: ash::Entry,
    instance: Instance,
    device: Device
}

impl Core {
    pub fn new(window: &Window) -> Core{
        let entry = unsafe {
            ash::Entry::load().expect("Failed to load the ash entry")
        };

        let instance = Instance::new(&entry, window);
        let device = Device::new(&instance.get_ash_instance());

        Core { entry, instance, device }
    }

    pub fn get_instance(&self) -> &Instance {
        &self.instance
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_entry(&self) -> &ash::Entry {
        &self.entry
    }
}