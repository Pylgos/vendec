use std::sync::Arc;

use ash::vk;

use crate::Result;

use super::{Image, Queue};

pub trait CommandBufferResource {}
impl<T> CommandBufferResource for T {}

pub struct CommandBuffer {
    pub(super) handle: vk::CommandBuffer,
    pub(super) queue: Arc<Queue>,
    resources: Vec<Arc<dyn CommandBufferResource>>,
}

pub struct Fence {
    handle: vk::Fence,
    command_buffer: CommandBuffer,
}

impl CommandBuffer {
    pub fn new(queue: Arc<Queue>) -> Result<Self> {
        let guard = queue.command_pool_lock.lock().unwrap();
        let info = vk::CommandBufferAllocateInfo::default()
            .command_pool(queue.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let handle = unsafe { queue.device.handle.allocate_command_buffers(&info)?[0] };
        let info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { queue.device.handle.begin_command_buffer(handle, &info)? };
        drop(guard);
        Ok(Self {
            handle,
            queue,
            resources: Vec::new(),
        })
    }

    pub fn lock_pool(&self) -> impl Drop + '_ {
        self.queue.command_pool_lock.lock().unwrap()
    }

    pub fn image_barrier(
        &mut self,
        image: Arc<Image>,
        src: vk::ImageLayout,
        dst: vk::ImageLayout,
        src_stage_mask: vk::PipelineStageFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
        src_access_mask: vk::AccessFlags2,
        dst_access_mask: vk::AccessFlags2,
        aspect_mask: vk::ImageAspectFlags,
        queue_transfer: Option<(&Queue, &Queue)>,
    ) {
        let (src_queue, dst_queue) = queue_transfer.map_or(
            (vk::QUEUE_FAMILY_IGNORED, vk::QUEUE_FAMILY_IGNORED),
            |(src, dst)| (src.family_index, dst.family_index),
        );
        let barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src_stage_mask)
            .dst_stage_mask(dst_stage_mask)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(src)
            .new_layout(dst)
            .src_queue_family_index(src_queue)
            .dst_queue_family_index(dst_queue)
            .image(image.image_handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect_mask)
                    .level_count(1)
                    .layer_count(1),
            );
        let barriers = [barrier];
        let dependency = vk::DependencyInfoKHR::default().image_memory_barriers(&barriers);
        unsafe {
            let _guard = self.lock_pool();
            self.queue
                .device
                .handle
                .cmd_pipeline_barrier2(self.handle, &dependency);
        };
        self.resources.push(image);
    }

    pub unsafe fn submit(self) -> Result<Fence> {
        unsafe {
            self.queue.device.handle.end_command_buffer(self.handle)?;
        }
        let fence = vk::FenceCreateInfo::default();
        let fence = unsafe { self.queue.device.handle.create_fence(&fence, None)? };
        let command_buffers = [self.handle];
        let submit = vk::SubmitInfo::default().command_buffers(&command_buffers);
        unsafe {
            let _guard = self.queue.queue_lock.lock().unwrap();
            self.queue
                .device
                .handle
                .queue_submit(self.queue.handle, &[submit], fence)?;
        }
        Ok(Fence {
            handle: fence,
            command_buffer: self,
        })
    }

    pub(super) fn add_resource(&mut self, resource: Arc<dyn CommandBufferResource>) {
        self.resources.push(resource);
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        let _guard = self.lock_pool();
        unsafe {
            self.queue
                .device
                .handle
                .free_command_buffers(self.queue.command_pool, &[self.handle]);
        }
    }
}

impl Fence {
    pub fn wait(&self) -> Result<()> {
        unsafe {
            self.command_buffer.queue.device.handle.wait_for_fences(
                &[self.handle],
                true,
                u64::MAX,
            )?;
        }
        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.command_buffer
                .queue
                .device
                .handle
                .destroy_fence(self.handle, None);
        }
    }
}
