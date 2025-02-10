use std::sync::Arc;

use ash::vk;

use super::Device;

#[derive(Debug)]
pub struct Memory {
    pub handle: vk::DeviceMemory,
    pub device: Arc<Device>,
    pub size: vk::DeviceSize,
    pub memory_type_index: u32,
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.free_memory(self.handle, None);
        }
    }
}
