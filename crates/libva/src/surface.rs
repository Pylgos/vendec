use paste::paste;
use std::sync::Arc;

use crate::{
    sys, va_bitflags, ConfigAttributes, Display, Fourcc, Image, Library, RtFormat, VaResult,
    VaStatusExt,
};

pub struct Surface {
    handle: sys::VASurfaceID,
    display: Arc<Display>,
}

impl Surface {
    pub fn new_many(
        display: Arc<Display>,
        format: RtFormat,
        width: u32,
        height: u32,
        pixel_format: Option<Fourcc>,
        usage_hint: UsageHint,
        count: u32,
    ) -> VaResult<Vec<Arc<Self>>> {
        let mut raw_attributes_list = Vec::new();
        if let Some(pixel_format) = pixel_format {
            raw_attributes_list.push(sys::VASurfaceAttrib {
                type_: sys::VASurfaceAttribPixelFormat,
                flags: sys::VA_SURFACE_ATTRIB_SETTABLE,
                value: pixel_format.to_raw(),
            });
        }
        raw_attributes_list.push(sys::VASurfaceAttrib {
            type_: sys::VASurfaceAttribUsageHint,
            flags: sys::VA_SURFACE_ATTRIB_SETTABLE,
            value: usage_hint.to_raw(),
        });
        let mut handles = vec![0; count as usize];
        unsafe {
            display
                .library()
                .lib()
                .vaCreateSurfaces(
                    display.handle(),
                    format.bits(),
                    width,
                    height,
                    handles.as_mut_ptr(),
                    count,
                    raw_attributes_list.as_mut_ptr(),
                    raw_attributes_list.len() as _,
                )
                .va_result()?;
        };
        handles
            .into_iter()
            .map(|handle| {
                Ok(Arc::new(Self {
                    handle,
                    display: display.clone(),
                }))
            })
            .collect()
    }

    pub fn new(
        display: Arc<Display>,
        format: RtFormat,
        width: u32,
        height: u32,
        pixel_format: Option<Fourcc>,
        usage_hint: UsageHint,
    ) -> VaResult<Arc<Self>> {
        Ok(
            Self::new_many(display, format, width, height, pixel_format, usage_hint, 1)?
                .into_iter()
                .next()
                .unwrap(),
        )
    }

    pub fn handle(&self) -> sys::VASurfaceID {
        self.handle
    }

    pub fn library(&self) -> &Arc<Library> {
        self.display.library()
    }

    pub fn display(&self) -> &Arc<Display> {
        &self.display
    }

    pub fn derive_image(&self) -> VaResult<Arc<Image>> {
        let mut raw_image = sys::VAImage::default();
        unsafe {
            self.library()
                .lib()
                .vaDeriveImage(self.display().handle(), self.handle(), &mut raw_image)
                .va_result()?;
        }
        Ok(Image::from_raw(self.display().clone(), raw_image))
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.library().lib().vaDestroySurfaces(
                self.display().handle(),
                &self.handle() as *const _ as *mut _,
                1,
            );
        }
    }
}

trait GenericValue: Sized {
    unsafe fn from_raw(raw: sys::VAGenericValue) -> Option<Self>;
    fn to_raw(self) -> sys::VAGenericValue;
}

impl GenericValue for i32 {
    unsafe fn from_raw(raw: sys::VAGenericValue) -> Option<Self> {
        if raw.type_ == sys::VAGenericValueTypeInteger {
            Some(raw.value.i)
        } else {
            None
        }
    }

    fn to_raw(self) -> sys::VAGenericValue {
        sys::VAGenericValue {
            type_: sys::VAGenericValueTypeInteger,
            value: sys::_VAGenericValue__bindgen_ty_1 { i: self },
        }
    }
}

impl GenericValue for u32 {
    unsafe fn from_raw(raw: sys::VAGenericValue) -> Option<Self> {
        if raw.type_ == sys::VAGenericValueTypeInteger {
            Some(raw.value.i as u32)
        } else {
            None
        }
    }

    fn to_raw(self) -> sys::VAGenericValue {
        sys::VAGenericValue {
            type_: sys::VAGenericValueTypeInteger,
            value: sys::_VAGenericValue__bindgen_ty_1 { i: self as i32 },
        }
    }
}

impl GenericValue for Fourcc {
    unsafe fn from_raw(raw: sys::VAGenericValue) -> Option<Self> {
        if raw.type_ == sys::VAGenericValueTypeInteger {
            Some(Fourcc::from(raw.value.i as u32))
        } else {
            None
        }
    }

    fn to_raw(self) -> sys::VAGenericValue {
        sys::VAGenericValue {
            type_: sys::VAGenericValueTypeInteger,
            value: sys::_VAGenericValue__bindgen_ty_1 {
                i: u32::from(self) as i32,
            },
        }
    }
}

macro_rules! va_surface_attribs {
    {$name:ident ; $($type_name:ident : $attrib_name:ident : $attrib_type:ty ,)*} => {
        #[derive(Debug, Clone, Eq, PartialEq, Default)]
        pub struct $name {
            $(
                pub $attrib_name: Option<$attrib_type>,
            )*
            pub pixel_formats: Vec<Fourcc>,
        }

        paste! {
            impl $name {
                pub fn empty_raw_attrib_list(size: usize) -> Vec<sys::VASurfaceAttrib> {
                    vec![sys::VASurfaceAttrib::default(); size]
                }

                // pub fn to_raw_attrib_list(&self) -> Vec<sys::VASurfaceAttrib> {
                //     let mut result = Vec::new();
                //     $(
                //         if let Some(&attr) = self.$attrib_name.as_ref() {
                //             result.push(sys::VASurfaceAttrib {
                //                 type_: sys::[<VASurfaceAttrib $type_name>],
                //                 flags: 0,
                //                 value: GenericValue::to_raw(attr),
                //             });
                //         }
                //     )*
                //     result
                // }

                pub unsafe fn from_raw_attrib_list(attrs: &[sys::VASurfaceAttrib]) -> Self {
                    let mut result = Self::default();
                    for attr in attrs {
                        match attr.type_ {
                            $(
                                sys::[<VASurfaceAttrib $type_name>] => {
                                    if attr.flags != sys::VA_SURFACE_ATTRIB_NOT_SUPPORTED {
                                        result.$attrib_name = <$attrib_type as GenericValue>::from_raw(attr.value);
                                    }
                                }
                            )*
                            sys::VASurfaceAttribPixelFormat => {
                                if attr.flags != sys::VA_SURFACE_ATTRIB_NOT_SUPPORTED {
                                    result.pixel_formats.push(Fourcc::from_raw(attr.value).unwrap());
                                }
                            }
                            _ => {}
                        }
                    }
                    result
                }
            }
        }
    };
}

macro_rules! va_surface_bitflags {
    {$name:ident; $prefix:ident { $($elem_name:ident ,)* } } => {
        ::paste::paste! {
            ::bitflags::bitflags! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    pub struct $name: u32 {
                        $(
                            const $elem_name = sys::[<$prefix $elem_name>] as u32;
                        )*
                        const _ = !0;
                    }
            }
        }

        impl GenericValue for $name {
            unsafe fn from_raw(raw: sys::VAGenericValue) -> Option<Self> {
                if raw.type_ == sys::VAGenericValueTypeInteger {
                    Some(Self::from_bits_truncate(raw.value.i as _))
                } else {
                    None
                }
            }

            fn to_raw(self) -> sys::VAGenericValue {
                sys::VAGenericValue {
                    type_: sys::VAGenericValueTypeInteger,
                    value: sys::_VAGenericValue__bindgen_ty_1 { i: self.bits() as _ },
                }
            }
        }
    }
}

va_surface_bitflags! {
    UsageHint;
    VA_SURFACE_ATTRIB_USAGE_HINT_ {
        GENERIC,
        DECODER,
        ENCODER,
        VPP_READ,
        VPP_WRITE,
        DISPLAY,
        EXPORT,
    }
}

va_surface_attribs! {
    SurfaceAttributes;
    MinWidth: min_width: u32,
    MaxWidth: max_width: u32,
    MinHeight: min_height: u32,
    MaxHeight: max_height: u32,
    UsageHint: usage_hint: UsageHint,
}
