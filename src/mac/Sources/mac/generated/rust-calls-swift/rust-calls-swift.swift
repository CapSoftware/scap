@_cdecl("__swift_bridge__$get_aperture_devices")
func __swift_bridge__get_aperture_devices () -> UnsafeMutableRawPointer {
    { let rustString = get_aperture_devices().intoRustString(); rustString.isOwned = false; return rustString.ptr }()
}



