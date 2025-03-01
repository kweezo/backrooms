use ash::{vk, ext, Entry};

use std::ffi::CStr;

unsafe extern "system" fn vulkan_debug_utils_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    if (*p_callback_data).message_id_number == -937765618 {
        return vk::FALSE;
    }

    let message = CStr::from_ptr((*p_callback_data).p_message);
    eprintln!("{:?}", message);
    panic!("everythings gone to shit lmfao");
}


pub fn create_debug_messenger(
    entry: &Entry,
    instance: &ash::Instance,
) -> (vk::DebugUtilsMessengerEXT, ext::debug_utils::Instance) {
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        ..Default::default()
    };

    let debug_messenger_instance = ext::debug_utils::Instance::new(entry, instance);
    let debug_messenger =
        unsafe { debug_messenger_instance.create_debug_utils_messenger(&create_info, None) }
            .expect("Failed to create the debug utils messenger");

    (debug_messenger, debug_messenger_instance)
}