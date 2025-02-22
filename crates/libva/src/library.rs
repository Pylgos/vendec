use std::sync::Arc;

use crate::sys;

pub struct Library {
    lib: sys::va,
}

impl Library {
    pub fn load() -> Result<Arc<Self>, libloading::Error> {
        let lib = unsafe { sys::va::new(libloading::library_filename("va-drm"))? };
        Ok(Arc::new(Self { lib }))
    }

    pub fn lib(&self) -> &sys::va {
        &self.lib
    }
}
