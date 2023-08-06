fn main() {
    let devices = ffi::get_aperture_devices();
    println!("Devices: {}", devices);
}

#[swift_bridge::bridge]
mod ffi {
    // extern "Rust" {
    //     fn rust_double_number(num: i64) -> i64;
    // }

    extern "Swift" {
        fn get_aperture_devices() -> String;
    }
}

// fn rust_double_number(num: i64) -> i64 {
//     println!("Rust double function called...");

//     num * 2
// }
