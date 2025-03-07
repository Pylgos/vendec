use paste::paste;
use std::sync::Arc;

use crate::{
    sys, va_bitflags, ConfigAttribValue, Display, Entrypoint, Library, Profile, RtFormat,
    SurfaceAttributes, VaResult, VaStatusExt,
};

pub struct Config {
    handle: sys::VAConfigID,
    display: Arc<Display>,
}

impl Config {
    pub fn new(
        display: Arc<Display>,
        profile: Option<Profile>,
        entrypoint: Entrypoint,
        attributes: &ConfigAttributes,
    ) -> VaResult<Arc<Self>> {
        let raw_profile = profile.map(Into::into).unwrap_or(sys::VAProfileNone);
        let raw_entrypoint = entrypoint.into();
        let mut raw_attrib_list = attributes.to_raw_attrib_list();
        let mut handle = sys::VAConfigID::default();
        unsafe {
            display
                .library()
                .lib()
                .vaCreateConfig(
                    display.handle(),
                    raw_profile,
                    raw_entrypoint,
                    raw_attrib_list.as_mut_ptr(),
                    raw_attrib_list.len() as _,
                    &mut handle,
                )
                .va_result()?;
        };
        Ok(Arc::new(Self { handle, display }))
    }

    pub fn handle(&self) -> sys::VAConfigID {
        self.handle
    }

    pub fn display(&self) -> &Arc<Display> {
        &self.display
    }

    pub fn library(&self) -> &Arc<Library> {
        self.display.library()
    }

    pub fn query_surface_attributes(&self) -> VaResult<SurfaceAttributes> {
        let mut num_attribs = 0;
        unsafe {
            self.library()
                .lib()
                .vaQuerySurfaceAttributes(
                    self.display().handle(),
                    self.handle(),
                    std::ptr::null_mut(),
                    &mut num_attribs,
                )
                .va_result()?
        };
        let mut attrib_list = Vec::with_capacity(num_attribs as usize);
        for _ in 0..num_attribs {
            attrib_list.push(sys::VASurfaceAttrib {
                type_: 0,
                flags: 0,
                value: sys::VAGenericValue {
                    type_: sys::VAGenericValueTypeInteger,
                    value: sys::_VAGenericValue__bindgen_ty_1 { i: 0 },
                },
            });
        }
        unsafe {
            self.library()
                .lib()
                .vaQuerySurfaceAttributes(
                    self.display().handle(),
                    self.handle(),
                    attrib_list.as_mut_ptr(),
                    &mut num_attribs,
                )
                .va_result()?;
        }
        Ok(unsafe { SurfaceAttributes::from_raw_attrib_list(&attrib_list) })
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            self.library()
                .lib()
                .vaDestroyConfig(self.display().handle(), self.handle());
        }
    }
}

macro_rules! va_config_attrib {
    {$name:ident ; $($type_name:ident : $attrib_name:ident : $attrib_type:ty ,)*} => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
        pub struct $name {
            $(
                pub $attrib_name: Option<$attrib_type>,
            )*
        }

        paste! {
            impl $name {
                pub fn default_raw_attrib_list() -> Vec<sys::VAConfigAttrib> {
                    vec![
                        $(
                            sys::VAConfigAttrib {
                                type_: sys::[<VAConfigAttrib $type_name>],
                                value: 0,
                            },
                        )*
                    ]
                }

                pub fn to_raw_attrib_list(&self) -> Vec<sys::VAConfigAttrib> {
                    let mut result = Vec::new();
                    $(
                        if let Some(&attr) = self.$attrib_name.as_ref() {
                            result.push(sys::VAConfigAttrib {
                                type_: sys::[<VAConfigAttrib $type_name>],
                                value: ConfigAttribValue::to_raw(attr),
                            });
                        }
                    )*
                    result
                }

                pub fn from_raw_attrib_list(attrs: &[sys::VAConfigAttrib]) -> Self {
                    let mut result = Self::default();
                    for attr in attrs {
                        match attr.type_ {
                            $(
                                sys::[<VAConfigAttrib $type_name>] => {
                                    if attr.value != sys::VA_ATTRIB_NOT_SUPPORTED {
                                        result.$attrib_name = <$attrib_type as ConfigAttribValue>::from_raw(attr.value);
                                    }
                                }
                            )*
                            _ => {}
                        }
                    }
                    result
                }


            }
        }
    };
}

va_bitflags! {
    DecSliceMode;
    VA_DEC_SLICE_MODE_ {
        NORMAL,
        BASE,
    }
}

va_bitflags! {
    EncPackedHeaders;
    VA_ENC_PACKED_HEADER_ {
        SEQUENCE,
        PICTURE,
        SLICE,
        MISC,
        RAW_DATA,
    }
}

va_bitflags! {
    EncInterlaced;
    VA_ENC_INTERLACED_ {
        FRAME,
        FIELD,
        MBAFF,
        PAFF,
    }
}

va_bitflags! {
    EncSliceStructure;
    VA_ENC_SLICE_STRUCTURE_ {
        POWER_OF_TWO_ROWS,
        ARBITRARY_MACROBLOCKS,
        EQUAL_ROWS,
        MAX_SLICE_SIZE,
        ARBITRARY_ROWS,
        EQUAL_MULTI_ROWS,
    }
}

va_bitflags! {
    EncQuantization;
    VA_ENC_QUANTIZATION_ {
        TRELLIS_SUPPORTED,
    }
}

va_bitflags! {
    EncIntraRefresh;
    VA_ENC_INTRA_REFRESH_ {
        ROLLING_COLUMN,
        ROLLING_ROW,
        ADAPTIVE,
        CYCLIC,
        P_FRAME,
        B_FRAME,
        MULTI_REF,
    }
}

// https://intel.github.io/libva/group__api__core.html#ga2c3be94ce142fb92a4bf93e9b1b4fa01
va_config_attrib! {
    ConfigAttributes;
    RTFormat: rt_format: RtFormat,
    DecSliceMode: dec_slice_mode: DecSliceMode,
    DecProcessing: dec_processing: bool,
    EncPackedHeaders: enc_packed_headers: EncPackedHeaders,
    EncInterlaced: enc_interlaced: EncInterlaced,
    EncMaxRefFrames: enc_max_ref_frames: u32,
    EncMaxSlices: enc_max_slices: u32,
    EncSliceStructure: enc_slice_structure: EncSliceStructure,
    EncMacroblockInfo: enc_macroblock_info: u32,
    MaxPictureWidth: max_picture_width: u32,
    MaxPictureHeight: max_picture_height: u32,
    EncQualityRange: enc_quality_range: u32,
    EncQuantization: enc_quantization: EncQuantization,
    EncIntraRefresh: enc_intra_refresh: EncIntraRefresh,
    EncSkipFrame: enc_skip_frame: bool,
}
