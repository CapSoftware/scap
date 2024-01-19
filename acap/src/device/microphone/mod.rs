use cpal::{traits::HostTrait, InputDevices, DevicesError, Devices};

pub fn get_targets() -> Result<InputDevices<Devices>, DevicesError> {
    let host = cpal::default_host();
    return host.input_devices(); 
}