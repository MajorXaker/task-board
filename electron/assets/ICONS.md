# Icons

Place your app icons here before building:

| File | Size | Used for |
|---|---|---|
| `icon.png` | 512×512 px (or 1024×1024) | Linux AppImage / deb / rpm |
| `icon.icns` | macOS icon bundle | macOS dmg / zip |
| `icon.ico` | Multi-size Windows icon | Windows NSIS / portable |

## Generating from a single PNG

If you have a 1024×1024 `source.png`:

```bash
# macOS icns (requires Xcode)
mkdir icon.iconset
for s in 16 32 64 128 256 512; do
  sips -z $s $s source.png --out icon.iconset/icon_${s}x${s}.png
done
iconutil -c icns icon.iconset

# Windows ico (requires ImageMagick)
convert source.png -resize 256x256 \
  -define icon:auto-resize="256,128,64,48,32,16" icon.ico

# Linux — just rename the source
cp source.png icon.png
```

Or use the `electron-icon-builder` npm package:
```bash
npx electron-icon-builder --input=source.png --output=assets/
```
