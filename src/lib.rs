//! Cross Platform, Performant and High Quality screen recordings

pub mod capturer;
pub mod frame;
mod targets;
mod utils;

// Helper Methods
pub use targets::get_all_targets;
pub use targets::Target;
pub use targets::Window;
pub use targets::Display;
pub use utils::has_permission;
pub use utils::is_supported;
pub use utils::request_permission;
