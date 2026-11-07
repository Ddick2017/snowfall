# Snowfall

[Русская версия](README.ru.md)

A lightweight falling-snow overlay for your desktop on **Windows 10/11, macOS and Linux**.

- Snow falls over all windows and **settles on window title bars and on the taskbar / Dock**.
- **Sweep it away with the mouse.** The overlay is click-through, so your apps keep working normally.
- **Fullscreen aware:** the snow stops completely while a fullscreen app (video, game, F11) is active.
- **No window, no Dock/taskbar icon.** Snowfall lives in the system tray / menu bar, with *Pause*, *Launch at login* and *Quit*.
- Small footprint: a native Rust core, ~400 flakes drawn on a single canvas at 30 fps, windows polled a few times per second.

Snowfall is an independent project inspired by the macOS app Flake and is not affiliated with it.

## Download

Grab the latest build from the [Releases page](../../releases/latest).

| System | File |
|---|---|
| Windows 10 / 11 | `Snowfall_*_x64-setup.exe`, `Snowfall_*_x64_en-US.msi`, or `snowfall-*-windows-x64-portable.zip` |
| macOS (Apple Silicon) | `Snowfall_*_aarch64.dmg` |
| macOS (Intel) | `Snowfall_*_x64.dmg` |
| Linux (Debian / Ubuntu) | `snowfall_*_amd64.deb` |
| Linux (any distribution) | `Snowfall_*_amd64.AppImage` or `snowfall-*-linux-x86_64.tar.gz` |

The builds are not code-signed, so your OS will ask for confirmation the first time.

## Install and use

**Windows** – run the installer (or unzip the portable build and start `snowfall.exe`). If SmartScreen appears, choose *More info → Run anyway*. Windows 10 needs the WebView2 runtime (already included in Windows 11 and in current Windows 10 updates).

**macOS** – open the `.dmg`, drag *Snowfall* to *Applications*, then run once in Terminal:

```bash
xattr -dr com.apple.quarantine /Applications/Snowfall.app
```

**Debian / Ubuntu** – `sudo apt install ./snowfall_*_amd64.deb`, then start *Snowfall* from your application menu.

**AppImage** – `chmod +x Snowfall_*.AppImage && ./Snowfall_*.AppImage`

**tar.gz** – unpack it and run `./install-linux.sh` (installs to `~/.local/bin` and enables launch at login).

Once running, click the snowflake icon in the tray / menu bar:

- **Pause snow** – toggle the snowfall on and off.
- **Launch at login** – start Snowfall automatically when you sign in.
- **Quit Snowfall** – close it completely.

Starting Snowfall a second time does nothing; the running instance keeps going.

### Command line

```bash
snowfall --autostart-on     # launch at login
snowfall --autostart-off
snowfall --quit             # stop a running instance
```

## Build from source

Requirements: [Rust](https://rustup.rs), Node.js 20+, and on Linux:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev librsvg2-dev \
  libayatana-appindicator3-dev patchelf libx11-dev libxi-dev libxtst-dev
```

```bash
npm install
npx tauri icon app-icon.png   # generates the platform icons
npx tauri build               # installers end up in src-tauri/target/release/bundle/
```

Pushing a tag such as `v0.1.1` makes GitHub Actions build and publish all installers (see `.github/workflows/release.yml`).

## Known limitations

- Only the primary monitor is covered.
- **Linux:** needs X11 or XWayland (Snowfall sets `GDK_BACKEND=x11` itself). On a pure Wayland session, the cursor and windows are only visible for X11 apps. GNOME does not show tray icons by default; install the *AppIndicator* extension, or use `snowfall --quit` to stop it.
- **macOS:** the system may keep the overlay below the menu bar.
- Memory use is dominated by the system WebView (typically tens of MB).

## License

[MIT](LICENSE)
