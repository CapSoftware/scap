import AppKit
import AVFoundation
import CoreMediaIO

public struct Audio: Hashable, Codable {
		public let id: String
		public let name: String
}

func get_aperture_devices() -> String {
		let devices = AVCaptureDevice.devices(for: .audio).map {
				Audio(id: $0.uniqueID, name: $0.localizedName)
		}

		print(devices)

		return "Hello from Swift!"
}

