use ash::{Instance, vk};
use serde::Deserialize;

use std::{fs, ffi::CString};

#[derive(Deserialize, Debug)]
struct Config {
    pub required_extensions: Vec<String>,
    pub optional_extensions: Vec<String>,
}


pub(crate) fn get_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> (Vec<*const i8>, Vec<CString>) {
    let conf = load_config("conf/device.json");

    let required_extensions_cstr: Vec<CString> =
        get_supported_extensions(instance, physical_device, &conf.required_extensions);

    if required_extensions_cstr.len() != conf.required_extensions.len() {
        panic!("Error: not all required device extensions present");
    }

    let required_extensions_cstr_ptr: Vec<*const i8> = required_extensions_cstr
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    let optional_extensions_cstr =
        get_supported_extensions(instance, physical_device, &conf.optional_extensions);

    let optional_extensions_cstr_ptr: Vec<*const i8> = optional_extensions_cstr
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    let mut extensions_cstr_ptr = required_extensions_cstr_ptr.clone();
    extensions_cstr_ptr.extend(optional_extensions_cstr_ptr.iter());

    let mut extensions_cstr = required_extensions_cstr;
    extensions_cstr.extend(optional_extensions_cstr);

    (extensions_cstr_ptr, extensions_cstr)
}


fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path).expect("Failed to read the instance config file");

    serde_json::from_str(&contents).expect("Could not parse instance JSON config")
}

fn get_supported_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    extensions: &Vec<String>,
) -> Vec<CString> {
    let supported_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get enumerate instance extension properties")
    };

    let mut available_extensions: Vec<CString> = Vec::new();

    for supported_extension in &supported_extensions {
        for extension in extensions {
            let mut optional_extension_v: Vec<i8> =
                extension.chars().map(|c| c as i8).collect();

            optional_extension_v.resize(vk::MAX_EXTENSION_NAME_SIZE, 0);

            if supported_extension.extension_name == *optional_extension_v {
                available_extensions.push(
                    CString::new(extension.as_str()).expect("Failed to create a new CString"),
                );
            }
        }
    }

    available_extensions
}