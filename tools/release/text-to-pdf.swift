// 渲染单字中文为矢量 PDF（macOS 菜单栏模板图）。
// 用法: swift text-to-pdf.swift <文字> <输出.pdf> [字号] [字体名]
//
// 菜单栏图标要的是模板图（template image）：只有 alpha 通道有意义，macOS 会按
// 深浅色模式自动反色。所以这里全画纯黑、背景全透明——不要画彩色，也不要画底板。
import Foundation
import CoreGraphics
import CoreText
import AppKit

let args = CommandLine.arguments
guard args.count >= 3 else {
    FileHandle.standardError.write("用法: text-to-pdf.swift <文字> <输出.pdf> [字号] [字体]\n".data(using: .utf8)!)
    exit(1)
}
let text = args[1]
let outPath = args[2]
let pointSize = args.count > 3 ? (CGFloat(Double(args[3]) ?? 100)) : 100
let fontName = args.count > 4 ? args[4] : "PingFangSC-Semibold"

// 菜单栏高 22pt，字形实际可用高度约 16-18pt；留一点边距免得贴边。
let pad: CGFloat = 2
let canvas = CGFloat(pointSize + pad * 2)

var mediaBox = CGRect(x: 0, y: 0, width: canvas, height: canvas)
guard let ctx = CGContext(consumer: CGDataConsumer(url: URL(fileURLWithPath: outPath) as CFURL)!,
                          mediaBox: &mediaBox, nil) else {
    FileHandle.standardError.write("建不了 PDF 上下文\n".data(using: .utf8)!); exit(1)
}
ctx.beginPDFPage(nil)   // 页面尺寸取自 context 的 mediaBox

// 文字绘制：模板图只要 alpha，所以前景纯黑、alpha 全 1。
let font = CTFontCreateWithName(fontName as CFString, pointSize, nil)
let attributed = NSAttributedString(string: text, attributes: [
    .font: font,
    .foregroundColor: NSColor.black,
])
let line = CTLineCreateWithAttributedString(attributed)
// 汉字按 em 方块排版，光学 bounds 会因笔画的伸出而大于字身；居中用字体度量
// （ascender/descender 居中）更符合预期，字身正好占满画布减边距。
let ascent = CTFontGetAscent(font), descent = CTFontGetDescent(font)
let typographic = CGFloat(ascent - descent)
let x = (canvas - typographic) / 2
let y = (canvas - typographic) / 2 - CGFloat(descent)

ctx.textMatrix = .identity
ctx.translateBy(x: x, y: y)
CTLineDraw(line, ctx)

ctx.endPDFPage()
ctx.closePDF()

print(String(format: "「%@」 %@ %.0fpt → 字身高 %.1f，画布 %.0f×%.0f",
             text, fontName, pointSize, typographic, canvas, canvas))
print("已写 \(outPath)")
