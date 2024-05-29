use super::Target;

// On Linux, the target is selected when a Recorder is instanciated because this
// requires user interaction
pub fn get_all_targets() -> Vec<Target> {
    Vec::new()
}
