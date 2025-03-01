use serde::Deserialize;

use ash::{
    vk::MAX_EXTENSION_NAME_SIZE,
    Entry,
};

use std::{
    ffi::{CStr, CString},
    fs,
};

use crate::Window;

#[derive(Deserialize, Debug)]
struct Config {
    pub required_extensions: Vec<String>,
    pub optional_extensions: Vec<String>,
    pub validation_layers: Vec<String>,
    pub enable_layers: bool,
}

pub fn get_extensions_and_layers(window: &Window, entry: &Entry) -> (
    Vec<CString>, Vec<*const i8>, Vec<CString>, Vec<*const i8>
) {
    let conf = load_config("conf/instance.json");

    let (extensions_ptr, extensions) = get_extensions(&conf, window, entry);
    let (layers, layers_ptr) = get_supported_layers(entry, &conf.validation_layers);

    (layers, layers_ptr, extensions, extensions_ptr)
}

fn get_extensions(
    conf: &Config,
    window: &Window,
    entry: &Entry,
) -> (Vec<*const i8>, Vec<CString>) {
    let mut glfw_extensions = window
        .get_context()
        .get_required_instance_extensions()
        .expect("Failed to get list of the required GLFW instance extensions");

    glfw_extensions.extend(conf.required_extensions.iter().cloned());
    glfw_extensions.sort();
    glfw_extensions.dedup();

    let mut required_extensions_cstr: Vec<CString> =
        get_supported_extensions(entry, &glfw_extensions);

    required_extensions_cstr.sort();
    required_extensions_cstr.dedup();

    assert!(
        required_extensions_cstr.len() == glfw_extensions.len(),
        "Error: not all required instance extensions present"
    );

    let required_extensions_cstr_ptr: Vec<*const i8> = required_extensions_cstr
        .iter()
        .map(|s| s.as_ptr())
        .collect();

    let optional_extensions_cstr =
        get_supported_extensions(entry, &conf.optional_extensions);

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

fn get_supported_layers(entry: &Entry, layers: &Vec<String>) -> (Vec<CString>, Vec<*const i8>) {
    let available_layers = unsafe {
        entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumreate supported instance layers")
    };

    let mut supported_layers: Vec<CString> = Vec::new();
    let mut unsupported_layers: Vec<String> = Vec::new();

    for layer in layers {
        let mut found = false;

        for supported_layer in &available_layers {
            let mut layer_v: Vec<i8> = layer.chars().map(|c| c as i8).collect();

            layer_v.resize(MAX_EXTENSION_NAME_SIZE, 0);

            if supported_layer.layer_name == *layer_v {
                supported_layers.push(
                    CString::new(layer.as_str()).expect("Failed to create a new CString"),
                );
                found = true;
            }
        }

        if !found {
            unsupported_layers.push(layer.clone());
        }
    }

    let mut supported_layers_ptrs = Vec::<*const i8>::with_capacity(supported_layers.len());

    for layer in supported_layers.iter() {
        supported_layers_ptrs.push(
            layer.as_ptr() as *const i8
        );
    }

    (supported_layers, supported_layers_ptrs)
}

fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path).expect("Failed to read the instance config file");

    serde_json::from_str(&contents).expect("Could not parse instance JSON config")
}

//Recieves a vector of extensions and returns the ones that are supported
fn get_supported_extensions(entry: &Entry, extensions: &Vec<String>) -> Vec<CString> {
    let supported_extensions = unsafe {
        entry
            .enumerate_instance_extension_properties(None)
            .expect("Failed to get enumerate instance extension properties")
    };

    let mut available_extensions: Vec<CString> = Vec::new();

    for supported_extension in &supported_extensions {
        for extension in extensions {
            let mut optional_extension_v: Vec<i8> =
                extension.chars().map(|c| c as i8).collect();

            optional_extension_v.resize(MAX_EXTENSION_NAME_SIZE, 0);

            if supported_extension.extension_name == *optional_extension_v {
                available_extensions.push(
                    CString::new(extension.as_str()).expect("Failed to create a new CString"),
                );
            }
        }
    }

    available_extensions
}