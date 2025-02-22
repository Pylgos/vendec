use std::sync::Arc;

use crate::{sys, ConfigAttributes, Display, Entrypoint, Library, Profile, VaResult, VaStatusExt};

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
    ) -> VaResult<Self> {
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
        Ok(Self { handle, display })
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
}
