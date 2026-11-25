<p align="center"><img src="assets/logo.svg" width="96" alt="Notepad logo"></p>

# Notepad

A fast, simple notepad built with [GPUI](https://www.gpui.rs/), the UI framework behind Zed.

- Tabs, undo/redo, selection, word/line navigation, IME input
- Find and replace, go to line, word wrap, line numbers
- 12 VS Code style themes (Dark+, Light+, One Dark Pro, Dracula, Monokai, Nord, Tokyo Night, ...), or follow the system light/dark setting
- Any installed font, font size, recent files, drag and drop to open
- Drag tabs to reorder, middle-click to close
- Reopens the files you had open last time, with cursor positions
- Optional autosave (File > Autosave): saves every few seconds and when you switch apps
- Crash recovery for unsaved notes, close confirmation, "file changed on disk" warning

## Download

Get the latest build for your OS from the [Releases](../../releases) page.

macOS (Apple Silicon): `Notepad-*-macos-arm64.dmg`. Open it and drag Notepad to Applications.
Linux and Windows builds are not available yet; build from source.

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
