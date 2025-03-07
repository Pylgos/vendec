use bitflags::bitflags;
use paste::paste;

pub use vendec_libva_sys as sys;

mod buffer;
mod config;
mod context;
mod display;
mod error;
mod image;
mod library;
mod surface;
pub use buffer::*;
pub use config::*;
pub use context::*;
pub use display::*;
pub use error::*;
pub use image::*;
pub use library::*;
pub use surface::*;

macro_rules! va_enum_prefix {
    {$name:ident: $sys_type:ty; $prefix:ident { $($elem_name:ident ,)* } } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $elem_name,
            )*
        }

        paste! {
            impl TryFrom<$sys_type> for $name {
                type Error = ();
                fn try_from(value: $sys_type) -> Result<Self, Self::Error> {
                    match value {
                        $(
                            sys::[<$prefix $elem_name>] => Ok(Self::$elem_name),
                        )*
                        _ => Err(()),
                    }
                }
            }
            impl From<$name> for $sys_type {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$elem_name => sys::[<$prefix $elem_name>],
                        )*
                    }
                }
            }
        }
    }
}
pub(crate) use va_enum_prefix;

macro_rules! va_enum_prefix_suffix {
    {$name:ident: $sys_type:ty; $prefix:ident { $($elem_name:ident ,)* } $suffix:ident } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $elem_name,
            )*
        }

        ::paste::paste! {
            impl TryFrom<$sys_type> for $name {
                type Error = ();
                fn try_from(value: $sys_type) -> Result<Self, Self::Error> {
                    match value {
                        $(
                            sys::[<$prefix $elem_name $suffix>] => Ok(Self::$elem_name),
                        )*
                        _ => Err(()),
                    }
                }
            }
            impl From<$name> for $sys_type {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$elem_name => sys::[<$prefix $elem_name $suffix>],
                        )*
                    }
                }
            }
        }
    }
}
pub(crate) use va_enum_prefix_suffix;

macro_rules! va_bitflags {
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

        impl ConfigAttribValue for $name {
            fn from_raw(raw: u32) -> Option<Self> {
                Some(Self::from_bits_truncate(raw))
            }

            fn to_raw(self) -> u32 {
                self.bits() as u32
            }
        }
    }
}
pub(crate) use va_bitflags;

trait ConfigAttribValue: Sized {
    fn from_raw(attr: u32) -> Option<Self>;
    fn to_raw(self) -> u32;
}

impl ConfigAttribValue for u32 {
    fn from_raw(attr: u32) -> Option<Self> {
        Some(attr)
    }

    fn to_raw(self) -> u32 {
        self
    }
}

impl ConfigAttribValue for bool {
    fn from_raw(attr: u32) -> Option<Self> {
        Some(attr != 0)
    }

    fn to_raw(self) -> u32 {
        self as u32
    }
}

va_enum_prefix! {
    Profile: sys::VAProfile;
    VAProfile {
        MPEG2Simple,
        MPEG2Main,
        MPEG4Simple,
        MPEG4AdvancedSimple,
        MPEG4Main,
        H264Baseline,
        H264Main,
        H264High,
        VC1Simple,
        VC1Main,
        VC1Advanced,
        H263Baseline,
        JPEGBaseline,
        H264ConstrainedBaseline,
        VP8Version0_3,
        H264MultiviewHigh,
        H264StereoHigh,
        HEVCMain,
        HEVCMain10,
        VP9Profile0,
        VP9Profile1,
        VP9Profile2,
        VP9Profile3,
        HEVCMain12,
        HEVCMain422_10,
        HEVCMain422_12,
        HEVCMain444,
        HEVCMain444_10,
        HEVCMain444_12,
        HEVCSccMain,
        HEVCSccMain10,
        HEVCSccMain444,
        AV1Profile0,
        AV1Profile1,
        HEVCSccMain444_10,
        Protected,
        H264High10,
        VVCMain10,
        VVCMultilayerMain10,
    }
}

va_enum_prefix! {
    Entrypoint: sys::VAEntrypoint;
    VAEntrypoint {
        VLD,
        IZZ,
        IDCT,
        MoComp,
        Deblocking,
        EncSlice,
        EncPicture,
        EncSliceLP,
        VideoProc,
        FEI,
        Stats,
        ProtectedTEEComm,
        ProtectedContent,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ByteOrder {
    MsbFirst,
    LsbFirst,
}

impl TryFrom<u32> for ByteOrder {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            sys::VA_LSB_FIRST => Ok(Self::LsbFirst),
            sys::VA_MSB_FIRST => Ok(Self::MsbFirst),
            _ => Err(()),
        }
    }
}

impl From<ByteOrder> for u32 {
    fn from(value: ByteOrder) -> Self {
        match value {
            ByteOrder::LsbFirst => sys::VA_LSB_FIRST,
            ByteOrder::MsbFirst => sys::VA_MSB_FIRST,
        }
    }
}

va_bitflags! {
    RtFormat;
    VA_RT_FORMAT_ {
        YUV420,
        YUV422,
        YUV444,
        YUV411,
        YUV400,

        YUV420_10,
        YUV422_10,
        YUV444_10,

        YUV420_12,
        YUV422_12,
        YUV444_12,

        RGB16,
        RGB32,
        RGBP,
        RGB32_10,
        PROTECTED,
        RGB32_10BPP,
        YUV420_10BPP,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fourcc(u32);

impl TryFrom<&str> for Fourcc {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            return Err(());
        }
        let bytes = value.as_bytes();
        Ok(Self(
            (bytes[0] as u32)
                | ((bytes[1] as u32) << 8)
                | ((bytes[2] as u32) << 16)
                | ((bytes[3] as u32) << 24),
        ))
    }
}

impl From<u32> for Fourcc {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Fourcc> for u32 {
    fn from(value: Fourcc) -> Self {
        value.0
    }
}

impl std::fmt::Debug for Fourcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = [
            self.0 as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 24) as u8,
        ];
        String::from_utf8_lossy(&bytes).fmt(f)
    }
}

impl std::fmt::Display for Fourcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = [
            self.0 as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 24) as u8,
        ];
        String::from_utf8_lossy(&bytes).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let lib = library::Library::load().unwrap();
        let display = Display::enumerate(lib).next().unwrap();
        let profiles = display.query_config_profiles().unwrap();
        let config_attrs = display
            .get_config_attributes(None, Entrypoint::VideoProc)
            .unwrap();
        println!("{:?}", profiles);
        println!("{:?}", config_attrs);
        let config =
            Config::new(display.clone(), None, Entrypoint::VideoProc, &config_attrs).unwrap();
        let surface_attrs = config.query_surface_attributes().unwrap();
        println!("{:?}", surface_attrs);
        let surface = Surface::new(
            display.clone(),
            RtFormat::YUV420,
            1920,
            1080,
            None,
            UsageHint::GENERIC,
        )
        .unwrap();
        let image = surface.derive_image();
        println!("{:?}", image);
    }
}
