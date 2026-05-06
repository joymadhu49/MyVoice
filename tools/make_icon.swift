import AppKit
import CoreGraphics

func renderIcon(size: Int) -> Data? {
    let s = CGFloat(size)
    let cs = CGColorSpaceCreateDeviceRGB()
    guard let ctx = CGContext(
        data: nil,
        width: size, height: size,
        bitsPerComponent: 8, bytesPerRow: 0,
        space: cs,
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else { return nil }

    // Background: rounded rect with gradient
    let radius = s * 0.22
    let bgRect = CGRect(x: 0, y: 0, width: s, height: s)
    let path = CGPath(roundedRect: bgRect, cornerWidth: radius, cornerHeight: radius, transform: nil)
    ctx.saveGState()
    ctx.addPath(path)
    ctx.clip()
    let colors = [
        CGColor(red: 107/255, green: 78/255, blue: 240/255, alpha: 1.0),
        CGColor(red: 154/255, green: 124/255, blue: 255/255, alpha: 1.0),
    ] as CFArray
    let locations: [CGFloat] = [0.0, 1.0]
    if let grad = CGGradient(colorsSpace: cs, colors: colors, locations: locations) {
        ctx.drawLinearGradient(grad, start: CGPoint(x: 0, y: s), end: CGPoint(x: s, y: 0), options: [])
    }
    ctx.restoreGState()

    // Mic glyph (white)
    ctx.setStrokeColor(red: 1, green: 1, blue: 1, alpha: 1.0)
    ctx.setFillColor(red: 1, green: 1, blue: 1, alpha: 1.0)
    let lw = max(1.0, s * 0.06)
    ctx.setLineWidth(lw)
    ctx.setLineCap(.round)
    ctx.setLineJoin(.round)

    // Mic capsule body
    let capW = s * 0.26
    let capH = s * 0.40
    let capX = (s - capW) / 2
    let capY = s * 0.30
    let cap = CGRect(x: capX, y: capY, width: capW, height: capH)
    let capPath = CGPath(roundedRect: cap, cornerWidth: capW/2, cornerHeight: capW/2, transform: nil)
    ctx.addPath(capPath)
    ctx.fillPath()

    // Stand: U-shaped arc under
    let archRadius = s * 0.22
    let archCenter = CGPoint(x: s/2, y: capY)
    ctx.beginPath()
    ctx.addArc(center: archCenter, radius: archRadius, startAngle: .pi, endAngle: 0, clockwise: true)
    ctx.strokePath()

    // Stem
    let stemTop = capY - archRadius
    let stemBottom = stemTop - s * 0.10
    ctx.beginPath()
    ctx.move(to: CGPoint(x: s/2, y: stemTop))
    ctx.addLine(to: CGPoint(x: s/2, y: stemBottom))
    ctx.strokePath()

    // Base line
    let baseY = stemBottom
    let baseW = s * 0.18
    ctx.beginPath()
    ctx.move(to: CGPoint(x: s/2 - baseW/2, y: baseY))
    ctx.addLine(to: CGPoint(x: s/2 + baseW/2, y: baseY))
    ctx.strokePath()

    guard let cg = ctx.makeImage() else { return nil }
    let rep = NSBitmapImageRep(cgImage: cg)
    return rep.representation(using: .png, properties: [:])
}

func writePNG(size: Int, path: String) {
    guard let data = renderIcon(size: size) else { fatalError("render \(size) failed") }
    try! data.write(to: URL(fileURLWithPath: path))
    print("wrote \(path) (\(size)x\(size))")
}

let args = CommandLine.arguments
guard args.count >= 2 else {
    print("usage: swift make_icon.swift <out_dir>")
    exit(1)
}
let out = args[1]
try? FileManager.default.createDirectory(atPath: out, withIntermediateDirectories: true)

let sizes: [(Int, String)] = [
    (16, "icon_16x16.png"),
    (32, "icon_16x16@2x.png"),
    (32, "icon_32x32.png"),
    (64, "icon_32x32@2x.png"),
    (128, "icon_128x128.png"),
    (256, "icon_128x128@2x.png"),
    (256, "icon_256x256.png"),
    (512, "icon_256x256@2x.png"),
    (512, "icon_512x512.png"),
    (1024, "icon_512x512@2x.png"),
]
for (s, name) in sizes {
    writePNG(size: s, path: "\(out)/\(name)")
}

// Tauri PNGs
let extras: [(Int, String)] = [
    (32, "32x32.png"),
    (128, "128x128.png"),
    (256, "128x128@2x.png"),
    (1024, "icon.png"),
    (30, "Square30x30Logo.png"),
    (44, "Square44x44Logo.png"),
    (71, "Square71x71Logo.png"),
    (89, "Square89x89Logo.png"),
    (107, "Square107x107Logo.png"),
    (142, "Square142x142Logo.png"),
    (150, "Square150x150Logo.png"),
    (284, "Square284x284Logo.png"),
    (310, "Square310x310Logo.png"),
    (50, "StoreLogo.png"),
]
for (s, name) in extras {
    writePNG(size: s, path: "\(out)/\(name)")
}

print("done")
