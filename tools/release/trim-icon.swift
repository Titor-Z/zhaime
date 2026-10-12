// 从源 logo 裁掉四周的背景与投影，输出正方形、铺满画布的透明底 PNG。
// 用法: swift trim-icon.swift <输入.png> <输出目录>
// 背景是浅灰渐变、外加一圈投影；内容是那块黑色圆角方块。与其排除背景，
// 不如找暗像素：背景 ~230 的亮度、投影 ~200，只有方块本身低于 120。
import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

let args = CommandLine.arguments
guard args.count == 3,
      let src = CGImageSourceCreateWithURL(URL(fileURLWithPath: args[1]) as CFURL, nil),
      let image = CGImageSourceCreateImageAtIndex(src, 0, nil) else {
    FileHandle.standardError.write("无法读取 \(args[1])\n".data(using: .utf8)!)
    exit(1)
}

let w = image.width, h = image.height
let rowBytes = w * 4
var pixels = [UInt8](repeating: 0, count: rowBytes * h)
let cs = CGColorSpaceCreateDeviceRGB()
let ctx = CGContext(data: &pixels, width: w, height: h, bitsPerComponent: 8,
                    bytesPerRow: rowBytes, space: cs,
                    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.draw(image, in: CGRect(x: 0, y: 0, width: w, height: h))

// 缓冲区坐标原点在左下（draw 之后是这样），找出内容的包围盒。
func isContent(_ x: Int, _ y: Int) -> Bool {
    let o = y * rowBytes + x * 4
    let lum = (Int(pixels[o]) * 299 + Int(pixels[o + 1]) * 587 + Int(pixels[o + 2]) * 114) / 1000
    return lum < 120
}
var minX = w, maxX = -1, minY = h, maxY = -1
for y in Swift.stride(from: 0, to: h, by: 2) {
    for x in Swift.stride(from: 0, to: w, by: 2) where isContent(x, y) {
        minX = min(minX, x); maxX = max(maxX, x)
        minY = min(minY, y); maxY = max(maxY, y)
    }
}
guard maxX > minX, maxY > minY else {
    FileHandle.standardError.write("没找到内容\n".data(using: .utf8)!); exit(1)
}

// 取正方形：以内容中心为中心，边长 = max(宽, 高)，再夹回画布内。
let cw = maxX - minX + 1, ch = maxY - minY + 1
let side = max(cw, ch)
let cx = (minX + maxX) / 2, cy = (minY + maxY) / 2
let sqX = max(0, min(w - side, cx - side / 2))
let sqY = max(0, min(h - side, cy - side / 2))
let square = CGRect(x: sqX, y: sqY, width: side, height: side)

// CGImage 的原点在左上，缓冲区的在左下，翻转 y。
let cropRect = CGRect(x: Int(square.minX), y: h - Int(square.maxY), width: side, height: side)
guard let cropped = image.cropping(to: cropRect) else {
    FileHandle.standardError.write("裁切失败\n".data(using: .utf8)!); exit(1)
}

// 铺到 outSize×outSize 的透明画布上。
let outSize = 2048
var out = [UInt8](repeating: 0, count: outSize * outSize * 4)
let octx = CGContext(data: &out, width: outSize, height: outSize, bitsPerComponent: 8,
                     bytesPerRow: outSize * 4, space: cs,
                     bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
octx.interpolationQuality = .high
octx.draw(cropped, in: CGRect(x: 0, y: 0, width: outSize, height: outSize))

print(String(format: "内容 %d×%d → 正方形 %d×%d（起点 %d,%d）→ 铺满 %d", cw, ch, side, side, sqX, sqY, outSize))

let dest = CGImageDestinationCreateWithURL(
    URL(fileURLWithPath: "\(args[2])/logo-trimmed.png") as CFURL,
    UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, octx.makeImage()!, nil)
CGImageDestinationFinalize(dest)
print("已写 \(args[2])/logo-trimmed.png")
