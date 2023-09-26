import Foundation
import ScreenCaptureKit

public struct Audio: Hashable, Codable {
    public let id: String
    public let name: String
}

func get_aperture_devices() -> String {
    print("Swift function started!")

    // let content = SCShareableContent.excludingDesktopWindows(
    //     false,
    //     onScreenWindowsOnly: true
    // )
    // let devices = AVCaptureDevice.devices(for: .audio).map {
    // 		Audio(id: $0.uniqueID, name: $0.localizedName)
    // }

    // print(devices)

    // logMousePosition()
    // RunLoop.current.run()

    // let recorder = ScreenRecorder.init();
    return "Hello from Swift!"
}

