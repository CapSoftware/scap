use super::Target;

// TODO
pub fn is_supported() -> bool {
    true
    // false
}

// TODO
pub fn has_permission() -> bool {
    true
    // false
}

// On Linux, the target is selected when a Recorder is instanciated because this
// requires user interaction
pub fn get_targets() -> Vec<Target> {
    Vec::new()
}
