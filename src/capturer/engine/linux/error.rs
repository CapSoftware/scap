use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub struct LinCapError {
    msg: String,
}

impl Error for LinCapError {}

impl LinCapError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl Display for LinCapError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<pipewire::Error> for LinCapError {
    fn from(e: pipewire::Error) -> Self {
        Self::new(e.to_string())
    }
}
