# Ferramp

A cross-platform audio player in Rust that recreates the classic
skinnable two-point-x era desktop player: main window, graphical
equalizer and playlist editor, docked together, pixel for pixel. It
runs on Linux, Windows and macOS. Every line of code and every pixel of
the built-in skin was written from scratch, and it is free software
(MIT or Apache-2.0).

Load any classic `.wsz` skin, including the original base skin if you
have it, and the windows look exactly as they did in the original.

## What it does

- Main window with time display, scrolling song title, bitrate and
  sample rate, mono/stereo, spectrum analyser and oscilloscope, volume,
  balance, seek bar, transport buttons, shuffle, repeat and the clutter
  bar (options, always on top, file info, double size, visualisation).
- Window-shade mode for all three windows, double size, EasyMove,
  always on top.
- 10-band equalizer with preamp, the spline graph, presets
  (built-in library, auto-load presets per file, EQF import and export).
- Playlist editor: resizable in steps, the ADD, REM, SEL, MISC and LIST
  button menus, multiple selection, drag to reorder, sorting, M3U, M3U8
  and PLS, jump to file, bookmarks, HTML export.
- Windows dock to each other and snap to screen edges; moving the main
  window moves the docked group.
- MP3, Ogg Vorbis, FLAC, WAV, AIFF, AAC/M4A, ALAC and more (Symphonia),
  internet radio over HTTP(S) with stream titles.
- Classic skins: `.wsz`/`.zip` or folders, `viscolor.txt`,
  `pledit.txt`, `region.txt` (shaped windows) and skin cursors.
- The classic menus, keyboard shortcuts and tooltips.

## Building

```sh
cargo build --release
./target/release/ferramp [files, folders, playlists or URLs]
```

Linux needs the ALSA development files (`alsa-lib`) and an X11 or
XWayland display. On Wayland desktops Ferramp uses XWayland, because the
docked borderless windows must be positioned freely; `FERRAMP_WAYLAND=1`
selects native Wayland, where the window manager places the windows.

To cross-check a build without a C compiler for the target, disable the
TLS feature: `cargo check --no-default-features --target x86_64-pc-windows-gnu`.

## Skins

Put `.wsz` files in the skins folder (`~/.config/ferramp/Skins` on Linux,
`%APPDATA%\ferramp\Skins` on Windows, `~/Library/Application Support/ferramp/Skins`
on macOS) or use *Skins > Skin Browser* (Alt+S). Missing bitmaps fall back
to the built-in skin, the way classic skins always worked.

## Keyboard

| Key | Action |
| --- | --- |
| Z X C V B | Previous, Play, Pause, Stop, Next |
| Shift+V, Ctrl+V | Stop with fade-out, stop after current |
| Left, Right | Back / forward 5 seconds |
| Up, Down | Volume |
| L, Ctrl+L, Shift+L | Open file, location, folder |
| J, Ctrl+J | Jump to file, jump to time |
| R, S | Repeat, shuffle |
| Ctrl+T, Ctrl+D, Ctrl+A, Ctrl+E | Time mode, double size, always on top, EasyMove |
| Alt+W, Alt+E, Alt+G | Main window, playlist editor, equalizer |
| Alt+3, Alt+S, Ctrl+P | File info, skin browser, preferences |
| Alt+I, Ctrl+Alt+I | Add bookmark, edit bookmarks |
| Numpad 0-9 | The classic numpad bindings |

## Options

`ferramp --render DIR [--skin FILE] [files]` renders every window to PNG
files without opening a window. `FERRAMP_SCALE=2` doubles everything on
very high-resolution screens. Settings are kept in `ferramp.ini` in the
configuration folder (`FERRAMP_CONFIG_DIR` overrides it).

## Licence

MIT or Apache-2.0, at your option. See `NOTICE` for third-party credits.
