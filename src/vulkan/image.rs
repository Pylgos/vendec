use std::sync::Arc;

use ash::vk;

use super::{CommandBuffer, Device, Memory};
use crate::{vulkan::Buffer, Result};

pub struct Image {
    pub image_handle: vk::Image,
    pub view_handle: vk::ImageView,
    pub memory: Arc<Memory>,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Arc<Self>> {
        let info = vk::ImageCreateInfo::default()
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .image_type(vk::ImageType::TYPE_2D)
            .usage(usage)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image_handle = unsafe { device.handle.create_image(&info, None)? };
        let mem_reqs = unsafe { device.handle.get_image_memory_requirements(image_handle) };
        let memory = Memory::new_prefer_device_local(device.clone(), &mem_reqs)?;
        unsafe {
            device
                .handle
                .bind_image_memory(image_handle, memory.handle, 0)?;
        }
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        let view_handle = unsafe { device.handle.create_image_view(&view_info, None)? };
        Ok(Arc::new(Image {
            image_handle,
            view_handle,
            memory,
            width,
            height,
        }))
    }

    pub fn extent(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.memory.device
    }

    pub fn cmd_update(
        self: Arc<Self>,
        cmd_buf: &mut CommandBuffer,
        data: &[u8],
        dst_layout: vk::ImageLayout,
        dst_stage: vk::PipelineStageFlags2,
        dst_access: vk::AccessFlags2,
    ) -> Result<()> {
        assert_eq!(
            self.memory.device.handle.handle(),
            cmd_buf.queue.device.handle.handle()
        );
        let device = &cmd_buf.queue.device;
        let staging = Buffer::new(
            device.clone(),
            data.len() as _,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let mut staging_mem = unsafe { staging.memory.write()? };
        staging_mem.copy_from_slice(data);
        staging_mem.flush()?;
        drop(staging_mem);

        cmd_buf.image_barrier(
            self.clone(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::TRANSFER,
            vk::AccessFlags2::empty(),
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::ImageAspectFlags::COLOR,
            None,
        );

        staging.cmd_copy_to_image(cmd_buf, self.clone(), vk::ImageAspectFlags::COLOR);

        cmd_buf.image_barrier(
            self,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            dst_layout,
            vk::PipelineStageFlags2::TRANSFER,
            dst_stage,
            vk::AccessFlags2::TRANSFER_WRITE,
            dst_access,
            vk::ImageAspectFlags::COLOR,
            None,
        );

        Ok(())
    }

    pub fn cmd_download(
        self: Arc<Self>,
        cmd_buf: &mut CommandBuffer,
        src_layout: vk::ImageLayout,
        src_stage: vk::PipelineStageFlags2,
        src_access: vk::AccessFlags2,
    ) -> Result<Arc<Buffer>> {
        let dst_size = self.width * self.height * 4;
        let staging = Buffer::new_host_visible(
            self.device().clone(),
            dst_size as _,
            vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        cmd_buf.image_barrier(
            self.clone(),
            src_layout,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            src_stage,
            vk::PipelineStageFlags2::TRANSFER,
            src_access,
            vk::AccessFlags2::TRANSFER_READ,
            vk::ImageAspectFlags::COLOR,
            None,
        );

        self.clone().cmd_copy_to_buffer(
            cmd_buf,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            staging.clone(),
        )?;

        cmd_buf.image_barrier(
            self,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            src_layout,
            vk::PipelineStageFlags2::TRANSFER,
            src_stage,
            vk::AccessFlags2::TRANSFER_READ,
            src_access,
            vk::ImageAspectFlags::COLOR,
            None,
        );

        Ok(staging)
    }

    pub fn cmd_copy_to_buffer(
        self: Arc<Self>,
        cmd_buf: &mut CommandBuffer,
        image_layout: vk::ImageLayout,
        buffer: Arc<Buffer>,
    ) -> Result<()> {
        let copy = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_extent(self.extent());
        unsafe {
            let _guard = cmd_buf.lock_pool();
            self.device().handle.cmd_copy_image_to_buffer(
                cmd_buf.handle,
                self.image_handle,
                image_layout,
                buffer.handle,
                &[copy],
            );
        }
        cmd_buf.add_resource(self);
        cmd_buf.add_resource(buffer);
        Ok(())
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            let device = &self.memory.device.handle;
            device.destroy_image_view(self.view_handle, None);
            device.destroy_image(self.image_handle, None);
        }
    }
}
