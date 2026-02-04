/**
 * 生成最小可用的 icon.ico（32x32），供 Tauri Windows 构建使用
 */
const fs = require("fs");
const path = require("path");

const outDir = path.join(__dirname, "..", "icons");
const icoPath = path.join(outDir, "icon.ico");

const w = 32,
  h = 32;
const headerSize = 40;
const pixelSize = w * h * 4;
const andMaskSize = Math.ceil((w * h) / 8);
const imageSize = headerSize + pixelSize + andMaskSize;
const fileHeader = Buffer.alloc(6);
const dirEntry = Buffer.alloc(16);
const bmpHeader = Buffer.alloc(40);
const pixels = Buffer.alloc(pixelSize);
const andMask = Buffer.alloc(andMaskSize);

fileHeader.writeUInt16LE(0, 0);
fileHeader.writeUInt16LE(1, 2);
fileHeader.writeUInt16LE(1, 4);

dirEntry[0] = w;
dirEntry[1] = h;
dirEntry[2] = 0;
dirEntry[3] = 0;
dirEntry.writeUInt16LE(1, 4);
dirEntry.writeUInt16LE(32, 6);
dirEntry.writeUInt32LE(imageSize, 8);
dirEntry.writeUInt32LE(22, 12);

bmpHeader.writeUInt32LE(40, 0);
bmpHeader.writeInt32LE(w, 4);
bmpHeader.writeInt32LE(h * 2, 8);
bmpHeader.writeUInt16LE(1, 12);
bmpHeader.writeUInt16LE(32, 14);
bmpHeader.writeUInt32LE(0, 16);
bmpHeader.writeUInt32LE(pixelSize, 20);
bmpHeader.writeInt32LE(0, 24);
bmpHeader.writeInt32LE(0, 28);
bmpHeader.writeUInt32LE(0, 32);
bmpHeader.writeUInt32LE(0, 36);

for (let y = h - 1; y >= 0; y--) {
  for (let x = 0; x < w; x++) {
    const i = (y * w + x) * 4;
    pixels[i] = 0x40;
    pixels[i + 1] = 0x70;
    pixels[i + 2] = 0xa0;
    pixels[i + 3] = 255;
  }
}

if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });
const ico = Buffer.concat([fileHeader, dirEntry, bmpHeader, pixels, andMask]);
fs.writeFileSync(icoPath, ico);
console.log("Written:", icoPath);
