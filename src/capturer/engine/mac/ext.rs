use cidre::cg;
use core_graphics_helmer_fork::display::{CGDisplay, CGDisplayMode};

pub trait DirectDisplayIdExt {
    fn display_mode(&self) -> Option<CGDisplayMode>;
}

impl DirectDisplayIdExt for cg::DirectDisplayId {
    #[inline]
    fn display_mode(&self) -> Option<CGDisplayMode> {
        CGDisplay::new(self.0).display_mode()
    }
}
