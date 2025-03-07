use core::str;
use std::sync::Arc;

use crate::{sys, Buffer, ByteOrder, Display, Fourcc, Library, VaResult, VaStatusExt};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageFormat {
    pub fourcc: Fourcc,
    pub byte_order: Option<ByteOrder>,
    pub bits_per_pixel: u32,

    // for rgb formats
    pub depth: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32,
}

impl TryFrom<sys::VAImageFormat> for ImageFormat {
    type Error = ();
    fn try_from(value: sys::VAImageFormat) -> Result<Self, Self::Error> {
        Ok(Self {
            fourcc: Fourcc::from(value.fourcc),
            byte_order: ByteOrder::try_from(value.byte_order).ok(),
            bits_per_pixel: value.bits_per_pixel,
            depth: value.depth,
            red_mask: value.red_mask,
            green_mask: value.green_mask,
            blue_mask: value.blue_mask,
            alpha_mask: value.alpha_mask,
        })
    }
}

impl From<ImageFormat> for sys::VAImageFormat {
    fn from(value: ImageFormat) -> Self {
        Self {
            fourcc: value.fourcc.into(),
            byte_order: value.byte_order.map(|bo| bo.into()).unwrap_or(0),
            bits_per_pixel: value.bits_per_pixel,
            depth: value.depth,
            red_mask: value.red_mask,
            green_mask: value.green_mask,
            blue_mask: value.blue_mask,
            alpha_mask: value.alpha_mask,
            va_reserved: [0; 4],
        }
    }
}

impl std::fmt::Debug for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageFormat")
            .field("fourcc", &self.fourcc)
            .field("byte_order", &self.byte_order)
            .field("bits_per_pixel", &self.bits_per_pixel)
            .field("depth", &self.depth)
            .field("red_mask", &format_args!("{:#010x}", self.red_mask))
            .field("green_mask", &format_args!("{:#010x}", self.green_mask))
            .field("blue_mask", &format_args!("{:#010x}", self.blue_mask))
            .field("alpha_mask", &format_args!("{:#010x}", self.alpha_mask))
            .finish()
    }
}

pub struct Image {
    raw: sys::VAImage,
    format: ImageFormat,
    display: Arc<Display>,
    buffer: Buffer,
}

impl Image {
    pub fn new(
        display: Arc<Display>,
        format: &ImageFormat,
        width: u32,
        height: u32,
    ) -> VaResult<Arc<Self>> {
        let mut raw_format = sys::VAImageFormat::from(*format);
        let mut raw = sys::VAImage::default();
        unsafe {
            display
                .library()
                .lib()
                .vaCreateImage(
                    display.handle(),
                    &mut raw_format,
                    width as _,
                    height as _,
                    &mut raw,
                )
                .va_result()?;
        }
        Ok(Self::from_raw(display, raw))
    }

    pub fn from_raw(display: Arc<Display>, raw: sys::VAImage) -> Arc<Self> {
        let buffer = Buffer::from_raw(display.clone(), raw.buf, raw.data_size as usize, false);
        let format = ImageFormat::try_from(raw.format).unwrap();
        Arc::new(Self {
            raw,
            display,
            buffer,
            format,
        })
    }

    pub fn display(&self) -> &Arc<Display> {
        &self.display
    }

    pub fn library(&self) -> &Arc<Library> {
        self.display.library()
    }

    pub fn handle(&self) -> sys::VAImageID {
        self.raw.image_id
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn format(&self) -> &ImageFormat {
        &self.format
    }

    pub fn width(&self) -> u32 {
        self.raw.width as _
    }

    pub fn height(&self) -> u32 {
        self.raw.height as _
    }

    pub fn num_planes(&self) -> u32 {
        self.raw.num_planes as _
    }

    pub fn pitches(&self) -> &[u32] {
        &self.raw.pitches[..self.num_planes() as usize]
    }

    pub fn offsets(&self) -> &[u32] {
        &self.raw.offsets[..self.num_planes() as usize]
    }

    pub fn num_pallet_entries(&self) -> u32 {
        self.raw.num_palette_entries as _
    }

    pub fn entry_bytes(&self) -> u32 {
        self.raw.entry_bytes as _
    }

    pub fn component_order(&self) -> &str {
        let i8_slice = &self.raw.component_order[..self.entry_bytes() as usize];
        let u8_slice =
            unsafe { std::slice::from_raw_parts(i8_slice.as_ptr() as *const u8, i8_slice.len()) };
        str::from_utf8(u8_slice).unwrap()
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .display
                .library()
                .lib()
                .vaDestroyImage(self.display.handle(), self.raw.image_id)
                .va_result();
        }
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("handle", &self.raw.image_id)
            .field("display", &self.display)
            .field("buffer", &self.buffer)
            .field("format", &self.format)
            .field("width", &self.width())
            .field("height", &self.height())
            .field("num_planes", &self.num_planes())
            .field("pitches", &self.pitches())
            .field("offsets", &self.offsets())
            .field("num_pallet_entries", &self.num_pallet_entries())
            .field("entry_bytes", &self.entry_bytes())
            .field("component_order", &self.component_order())
            .finish()
    }
}
