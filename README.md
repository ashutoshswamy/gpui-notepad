# Notepad

A fast, simple notepad built with [GPUI](https://www.gpui.rs/), the UI framework behind Zed.

- Tabs, undo/redo, selection, word/line navigation, IME input
- Find and replace, go to line, word wrap, line numbers
- 12 VS Code style themes (Dark+, Light+, One Dark Pro, Dracula, Monokai, Nord, Tokyo Night, ...)
- Font family and size, recent files, drag and drop to open
- Crash recovery for unsaved notes, close confirmation, "file changed on disk" warning

## Download

Get the latest build for your OS from the [Releases](../../releases) page.

| OS | File |
|---|---|
| macOS (Apple Silicon) | `Notepad-*-macos-arm64.dmg` |
| Linux (x86_64) | `gpui-notepad-*-linux-x86_64.tar.gz` |
| Windows (x86_64) | `gpui-notepad-*-windows-x86_64.zip` |

### macOS first launch

The app is ad-hoc signed, not notarized by Apple. On first launch macOS may say it cannot be verified:
right-click `Notepad.app`, choose **Open**, then **Open** again. Or run
`xattr -dr com.apple.quarantine /Applications/Notepad.app`.

## Build from source

```
cargo run --release
```

macOS needs Xcode and its Metal toolchain (`xcodebuild -downloadComponent MetalToolchain`).
`scripts/bundle-macos.sh` produces the `.app` and `.dmg`.

## Shortcuts

| | |
|---|---|
| New / Open / Save / Save As / Close tab | Cmd+N / O / S / Shift+S / W |
| Find / Replace / Next / Previous | Cmd+F / Opt+F / G / Shift+G |
| Go to line | Cmd+L |
| Undo / Redo | Cmd+Z / Shift+Z |
| Zoom | Cmd+= / Cmd+- |
| Insert date and time | F5 |

On Linux and Windows use Ctrl instead of Cmd. Settings live in `~/.config/gpui-notepad/`.

## License

MIT
