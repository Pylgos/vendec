use std::sync::Arc;

use bitflags::bitflags;

use crate::{sys, Config, Display, Surface, VaResult, VaStatusExt};

pub struct Context {
    handle: sys::VAContextID,
    config: Arc<Config>,
    _targets: Vec<Arc<Surface>>,
}

impl Context {
    pub fn new(
        config: Arc<Config>,
        picture_width: u32,
        picture_height: u32,
        flags: ContextFlags,
        render_targets: Vec<Arc<Surface>>,
    ) -> VaResult<Arc<Self>> {
        let mut render_target_ids: Vec<_> = render_targets.iter().map(|s| s.handle()).collect();
        let display = config.display();
        let mut handle = 0;
        unsafe {
            display
                .library()
                .lib()
                .vaCreateContext(
                    config.display().handle(),
                    config.handle(),
                    picture_width as _,
                    picture_height as _,
                    flags.bits() as _,
                    render_target_ids.as_mut_ptr(),
                    render_target_ids.len() as _,
                    &mut handle,
                )
                .va_result()?;
        };
        Ok(Arc::new(Self {
            handle,
            config,
            _targets: render_targets,
        }))
    }

    pub fn handle(&self) -> sys::VAContextID {
        self.handle
    }

    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    pub fn display(&self) -> &Arc<Display> {
        self.config.display()
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.display()
                .library()
                .lib()
                .vaDestroyContext(self.display().handle(), self.handle())
                .va_result()
                .ok();
        }
    }
}

bitflags! {
    pub struct ContextFlags: u32 {
        const PROGRESSIVE  = sys::VA_PROGRESSIVE;
    }
}
