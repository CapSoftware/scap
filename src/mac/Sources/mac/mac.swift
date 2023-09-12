import Foundation
import AVFoundation
import CoreGraphics
import CoreMediaIO
import ScreenCaptureKit
import Cocoa

public struct Audio: Hashable, Codable {
		public let id: String
		public let name: String
}

func logMousePosition() {
    var counter = 0
    let duration = 10  // in seconds
    let interval = 0.333  // in seconds (3 times a second)

    let timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { timer in
        let mouseLocation = NSEvent.mouseLocation
        print("Time: \(counter * Int(interval)) seconds, Mouse Position: \(mouseLocation)")
        
        counter += 1
        if counter * Int(interval) >= duration {
            timer.invalidate()
        }
    }
    
    RunLoop.current.add(timer, forMode: .common)
}



func get_aperture_devices() -> String {
		print("Swift function started!")
		let devices = AVCaptureDevice.devices(for: .audio).map {
				Audio(id: $0.uniqueID, name: $0.localizedName)
		}

        print(devices)

		// logMousePosition()
		// RunLoop.current.run()

		// let recorder = ScreenRecorder.init();
		return "Hello from Swift!"
}

