use ash::{
    ext,
    vk,
    Entry,
};

use std::ffi::CStr;

use crate::Window;

use super::{debug_messenger, instance_config};


pub struct Instance {
    vk_instance: ash::Instance,

    debug_messenger_instance: ext::debug_utils::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Instance {



    pub fn new(entry: &Entry, window: &Window) -> Instance {

        let app_name: &CStr = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"backrooms\0") };

       let (layers, layers_ptr, extensions, extensions_ptr) =
        instance_config::get_extensions_and_layers(window, entry);

        #[allow(deprecated)]
        let app_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            application_version: vk::make_version(0, 0, 1),
            p_engine_name: app_name.as_ptr(),
            p_application_name: app_name.as_ptr(),
            ..Default::default()
        };

        let create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions_ptr)
            .enabled_layer_names(&layers_ptr);

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create a vkInstance")
        };

        let (debug_messenger, debug_messenger_instance) =
            debug_messenger::create_debug_messenger(entry, &instance);

        Instance {
            vk_instance: instance,

            debug_messenger_instance,
            debug_messenger,
        }
    }

    pub fn get_ash_instance(&self) -> &ash::Instance {
        &self.vk_instance
    }
}