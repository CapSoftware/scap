// swift-tools-version: 5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "mac",
    products: [
        .library(name: "mac", type: .static, targets: ["mac"]),
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "mac",
            dependencies: []
        )
    ]
)
