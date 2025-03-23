use ash::vk;
use crate::engine::Device;

use std::io::prelude::*;
use std::fs::File;

pub struct Shader<'a> {
    vert_module: vk::ShaderModule,
    frag_module: vk::ShaderModule,

    device: &'a Device
}

impl<'a> Shader<'a> {
    pub fn new(device: &'a Device, vert_path: &str, frag_path: &str) -> Shader<'a> {
        let vert_module = Shader::create_shader_module(device, vert_path);
        let frag_module = Shader::create_shader_module(device, frag_path);

        Shader { vert_module, frag_module, device }
    }

    fn read_file(path: &str) -> Vec<u32> {
        let mut file = File::open(path)
        .expect(format!("Failed to load shader file {}", path).as_str());
    
        let mut contents = Vec::new(); 

        file.read_to_end(&mut contents)
        .expect(format!("Failed to read the contents of shader file {} into buffer", path).as_str());

        let mut contents_u32 = Vec::<u32>::with_capacity(contents.len() / 4);

        for i in 0..(contents.len() / 4) {
            contents_u32.push(
                (contents[(i * 4) + 0] as u32) << 0  |
                (contents[(i * 4) + 1] as u32) << 8  |
                (contents[(i * 4) + 2] as u32) << 16 |
                (contents[(i * 4) + 3] as u32) << 24 
            );
        }

        contents_u32
    }

    fn create_shader_module(device: & Device, path: &str) -> vk::ShaderModule{
        let code = Shader::read_file(path);

        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,

            code_size: code.len() * 4,
            p_code: code.as_ptr(),

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_shader_module(&create_info, None)
            .expect(format!("Failed to create a shader module for {}", path).as_str())
        }
    }

    pub fn get_shader_modules(&self) -> (vk::ShaderModule, vk::ShaderModule) {
        (self.vert_module, self.frag_module)
    }

}


impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.get_ash_device().destroy_shader_module(self.vert_module, None);
            self.device.get_ash_device().destroy_shader_module(self.frag_module, None);
        }
    }
}