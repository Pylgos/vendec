use std::sync::Arc;

use crate::{sys, va_enum_prefix_suffix, Context, Display, VaResult, VaStatusExt};

#[derive(Debug)]
pub struct Buffer {
    handle: sys::VABufferID,
    size: usize,
    display: Arc<Display>,
    owned: bool,
    // context: Arc<Context>,
}

pub struct BufferMap<'a> {
    buffer: &'a Buffer,
    data: &'a mut [u8],
}

impl Buffer {
    pub fn new(context: Arc<Context>, buffer_type: BufferType, size: usize) -> VaResult<Arc<Self>> {
        let display = context.display().clone();
        let mut handle = 0;
        unsafe {
            display
                .library()
                .lib()
                .vaCreateBuffer(
                    display.handle(),
                    context.handle(),
                    buffer_type.into(),
                    size as _,
                    1,
                    std::ptr::null_mut(),
                    &mut handle,
                )
                .va_result()?;
        }
        Ok(Arc::new(Self {
            handle,
            size,
            display,
            owned: true,
        }))
    }

    pub fn new_with_data(
        context: Arc<Context>,
        buffer_type: BufferType,
        data: &[u8],
    ) -> VaResult<Arc<Self>> {
        let display = context.display().clone();
        let mut handle = 0;
        unsafe {
            display
                .library()
                .lib()
                .vaCreateBuffer(
                    display.handle(),
                    context.handle(),
                    buffer_type.into(),
                    data.len() as _,
                    1,
                    data.as_ptr() as *mut _,
                    &mut handle,
                )
                .va_result()?;
        }
        Ok(Arc::new(Self {
            handle,
            size: data.len(),
            display,
            owned: true,
        }))
    }

    pub fn from_raw(
        display: Arc<Display>,
        handle: sys::VABufferID,
        size: usize,
        owned: bool,
    ) -> Self {
        Self {
            display,
            handle,
            size,
            owned,
        }
    }

    pub fn display(&self) -> &Arc<Display> {
        &self.display
    }

    pub fn library(&self) -> &Arc<crate::Library> {
        self.display.library()
    }

    pub fn map(&self) -> VaResult<BufferMap<'_>> {
        let display = self.display();
        let mut data = std::ptr::null_mut();
        unsafe {
            display
                .library()
                .lib()
                .vaMapBuffer(display.handle(), self.handle, &mut data)
                .va_result()?;
        }
        Ok(BufferMap {
            buffer: self,
            data: unsafe { std::slice::from_raw_parts_mut(data as *mut u8, self.size) },
        })
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                self.library()
                    .lib()
                    .vaDestroyBuffer(self.display().handle(), self.handle);
            }
        }
    }
}

impl<'a> std::ops::Deref for BufferMap<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> std::ops::DerefMut for BufferMap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl Drop for BufferMap<'_> {
    fn drop(&mut self) {
        unsafe {
            self.buffer
                .library()
                .lib()
                .vaUnmapBuffer(self.buffer.display().handle(), self.buffer.handle);
        }
    }
}

va_enum_prefix_suffix! {
    BufferType: sys::VABufferType;
    VA {
        PictureParameter,
        IQMatrix,
        BitPlane,
        SliceGroupMap,
        SliceParameter,
        SliceData,
        MacroblockParameter,
        ResidualData,
        DeblockingParameter,
        Image,
        ProtectedSliceData,
        QMatrix,
        HuffmanTable,
        Probability,
        EncCoded,
        EncSequenceParameter,
        EncPictureParameter,
        EncSliceParameter,
        EncPackedHeaderParameter,
        EncPackedHeaderData,
        EncMiscParameter,
        EncMacroblockParameter,
        EncMacroblockMap,
        EncQP,
        ProcPipelineParameter,
        ProcFilterParameter,
        EncFEIMV,
        EncFEIMBCode,
        EncFEIDistortion,
        EncFEIMBControl,
        EncFEIMVPredictor,
        StatsStatisticsParameter,
        StatsStatistics,
        StatsStatisticsBottomField,
        StatsMV,
        StatsMVPredictor,
        EncMacroblockDisableSkipMap,
        EncFEICTBCmd,
        EncFEICURecord,
        DecodeStreamout,
        SubsetsParameter,
        ContextParameterUpdate,
        ProtectedSessionExecute,
        EncryptionParameter,
        EncDeltaQpPerBlock,
        Alf,
        Lmcs,
        SubPic,
        Tile,
        SliceStruct,
   } BufferType
}
