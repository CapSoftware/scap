pub mod capturer;
mod device;
pub mod frame;

// Helper Methods
pub use device::display::has_permission;
pub use device::display::is_supported;
pub use device::display::get_targets;