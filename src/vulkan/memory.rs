use std::sync::Arc;

use ash::vk;

use crate::{Error, Result};

use super::Device;

#[derive(Debug)]
pub struct Memory {
    pub(super) handle: vk::DeviceMemory,
    pub(super) device: Arc<Device>,
    pub(super) size: vk::DeviceSize,
    pub(super) memory_type_index: u32,
    mapped: Option<*mut u8>,
}

impl Memory {
    pub fn new(
        device: Arc<Device>,
        required_properties: vk::MemoryPropertyFlags,
        requirements: &vk::MemoryRequirements,
    ) -> Result<Arc<Self>> {
        for memory_type_index in 0..device.physical_device.memory_properties.memory_type_count {
            if requirements.memory_type_bits & (1 << memory_type_index) == 0 {
                continue;
            }
            let properties = device.physical_device.memory_properties.memory_types
                [memory_type_index as usize]
                .property_flags;
            if !properties.contains(required_properties) {
                continue;
            }
            let info = vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(memory_type_index);
            let handle = unsafe { device.handle.allocate_memory(&info, None)? };
            let mapped = if required_properties.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
                let mapped = unsafe {
                    device.handle.map_memory(
                        handle,
                        0,
                        requirements.size,
                        vk::MemoryMapFlags::empty(),
                    )?
                } as *mut u8;
                Some(mapped)
            } else {
                None
            };
            return Ok(Arc::new(Memory {
                handle,
                device,
                size: requirements.size,
                memory_type_index,
                mapped,
            }));
        }
        Err(Error::NoMatchingMemoryType)
    }

    pub fn new_prefer_device_local(
        device: Arc<Device>,
        requirements: &vk::MemoryRequirements,
    ) -> Result<Arc<Self>> {
        match Self::new(
            device.clone(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            requirements,
        ) {
            Ok(mem) => Ok(mem),
            Err(Error::NoMatchingMemoryType) => {
                Self::new(device, vk::MemoryPropertyFlags::empty(), requirements)
            }
            Err(err) => Err(err),
        }
    }

    // Safety: The caller must ensure that the memory is not written to.
    pub unsafe fn read(&self) -> Result<MemoryRead> {
        if let Some(mapped) = self.mapped {
            let range = vk::MappedMemoryRange::default()
                .memory(self.handle)
                .size(self.size)
                .offset(0);
            unsafe {
                self.device
                    .handle
                    .invalidate_mapped_memory_ranges(&[range])?;
            }
            Ok(MemoryRead {
                memory: self,
                data: unsafe { std::slice::from_raw_parts(mapped, self.size as usize) },
            })
        } else {
            Err(Error::Other(anyhow::anyhow!("Memory is not host visible")))
        }
    }

    // Safety:
    // - The caller must ensure that the memory is not in use.
    // - Content of the memory is undefined before writing.
    pub unsafe fn write(&self) -> Result<MemoryWrite> {
        if let Some(mapped) = self.mapped {
            Ok(MemoryWrite {
                memory: self,
                data: unsafe { std::slice::from_raw_parts_mut(mapped, self.size as usize) },
                flushed: false,
            })
        } else {
            Err(Error::Other(anyhow::anyhow!("Memory is not host visible")))
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.free_memory(self.handle, None);
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

pub struct MemoryRead<'a> {
    memory: &'a Memory,
    data: &'a [u8],
}

impl std::ops::Deref for MemoryRead<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

pub struct MemoryWrite<'a> {
    memory: &'a Memory,
    data: &'a mut [u8],
    flushed: bool,
}

impl<'a> MemoryWrite<'a> {
    pub fn flush(&mut self) -> Result<()> {
        unsafe {
            self.memory.device.handle.flush_mapped_memory_ranges(&[
                vk::MappedMemoryRange::default()
                    .memory(self.memory.handle)
                    .size(self.memory.size)
                    .offset(0),
            ])?;
        }
        self.flushed = true;
        Ok(())
    }
}

impl std::ops::Deref for MemoryWrite<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl std::ops::DerefMut for MemoryWrite<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl Drop for MemoryWrite<'_> {
    fn drop(&mut self) {
        if self.flushed {
            return;
        }
        match self.flush() {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to flush memory: {:?}", err);
            }
        }
    }
}
