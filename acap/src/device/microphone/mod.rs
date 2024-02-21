use cpal::{traits::HostTrait, Devices, DevicesError, InputDevices};

pub fn get_targets() -> Result<InputDevices<Devices>, DevicesError> {
    let host = cpal::default_host();
    return host.input_devices();
}
