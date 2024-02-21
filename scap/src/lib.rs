//! Cross Platform, Performant and High Quality screen recordings

pub mod capturer;
mod device;
pub mod frame;

// Helper Methods
pub use device::display::get_targets;
pub use device::display::has_permission;
pub use device::display::is_supported;
