use std::sync::Arc;

use super::{CommandBuffer, Device, Image};
use crate::Result;
use ash::vk;

pub struct Buffer {
    pub handle: vk::Buffer,
    pub memory: Arc<super::Memory>,
}

impl Buffer {
    pub fn new(
        device: Arc<Device>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_props: vk::MemoryPropertyFlags,
    ) -> Result<Arc<Self>> {
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let handle = unsafe { device.handle.create_buffer(&info, None)? };
        let mem_reqs = unsafe { device.handle.get_buffer_memory_requirements(handle) };
        let memory = super::Memory::new(device.clone(), mem_props, &mem_reqs)?;
        unsafe {
            device.handle.bind_buffer_memory(handle, memory.handle, 0)?;
        }
        Ok(Arc::new(Self { handle, memory }))
    }

    pub fn new_prefer_device_local(
        device: Arc<Device>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Arc<Self>> {
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let handle = unsafe { device.handle.create_buffer(&info, None)? };
        let mem_reqs = unsafe { device.handle.get_buffer_memory_requirements(handle) };
        let memory = super::Memory::new_prefer_device_local(device.clone(), &mem_reqs)?;
        unsafe {
            device.handle.bind_buffer_memory(handle, memory.handle, 0)?;
        }
        Ok(Arc::new(Self { handle, memory }))
    }

    pub fn new_host_visible(
        device: Arc<Device>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Arc<Self>> {
        Self::new(device, size, usage, vk::MemoryPropertyFlags::HOST_VISIBLE)
    }

    pub fn cmd_copy_to_image(
        self: Arc<Self>,
        cmd_buf: &mut CommandBuffer,
        image: Arc<Image>,
        aspect_mask: vk::ImageAspectFlags,
    ) {
        let copy = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(aspect_mask)
                    .layer_count(1),
            )
            .image_extent(image.extent());
        unsafe {
            let _guard = cmd_buf.lock_pool();
            self.memory.device.handle.cmd_copy_buffer_to_image(
                cmd_buf.handle,
                self.handle,
                image.image_handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy],
            );
        }
        cmd_buf.add_resource(image);
        cmd_buf.add_resource(self);
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.memory.device.handle.destroy_buffer(self.handle, None);
        }
    }
}
