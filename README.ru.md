# Snowfall

[English version](README.md)

Лёгкий оверлей с падающим снегом для рабочего стола на **Windows 10/11, macOS и Linux**.

- Снег идёт поверх всех окон и **ложится на заголовки окон и на панель задач / Dock**.
- **Снег можно смахнуть мышкой.** Клики проходят сквозь оверлей, программы работают как обычно.
- **Учитывает полноэкранный режим:** пока открыто полноэкранное приложение (видео, игра, F11), снег полностью останавливается.
- **Нет окна и значка в Dock / на панели задач.** Программа живёт в системном трее / строке меню: *Пауза*, *Запускать при входе*, *Выйти*.
- Лёгкая: ядро на Rust, около 400 снежинок на одном canvas, 30 кадров/с, список окон опрашивается несколько раз в секунду.

Snowfall — независимый проект, вдохновлённый macOS-приложением Flake, и не связан с ним.

## Скачать

Последняя версия — на странице [Releases](../../releases/latest).

| Система | Файл |
|---|---|
| Windows 10 / 11 | `Snowfall_*_x64-setup.exe`, `Snowfall_*_x64_en-US.msi` или `snowfall-*-windows-x64-portable.zip` |
| macOS (Apple Silicon) | `Snowfall_*_aarch64.dmg` |
| macOS (Intel) | `Snowfall_*_x64.dmg` |
| Linux (Debian / Ubuntu) | `snowfall_*_amd64.deb` |
| Linux (любой дистрибутив) | `Snowfall_*_amd64.AppImage` или `snowfall-*-linux-x86_64.tar.gz` |

Сборки не подписаны сертификатом, поэтому при первом запуске система попросит подтверждение.

## Установка и использование

**Windows** — запустите установщик (или распакуйте portable-версию и запустите `snowfall.exe`). Если появится SmartScreen, нажмите *Подробнее → Выполнить в любом случае*. На Windows 10 нужен WebView2 (в Windows 11 и в обновлённой Windows 10 он уже есть).

**macOS** — откройте `.dmg`, перетащите *Snowfall* в *Программы* и один раз выполните в Терминале:

```bash
xattr -dr com.apple.quarantine /Applications/Snowfall.app
```

**Debian / Ubuntu** — `sudo apt install ./snowfall_*_amd64.deb`, затем запустите *Snowfall* из меню приложений.

**AppImage** — `chmod +x Snowfall_*.AppImage && ./Snowfall_*.AppImage`

**tar.gz** — распакуйте и выполните `./install-linux.sh` (установит в `~/.local/bin` и включит автозапуск).

После запуска нажмите на значок снежинки в трее / строке меню:

- **Pause snow** — включить или выключить снегопад.
- **Launch at login** — запускать Snowfall при входе в систему.
- **Quit Snowfall** — полностью закрыть программу.

Повторный запуск ничего не делает: уже работающий экземпляр продолжает работать.

### Командная строка

```bash
snowfall --autostart-on     # автозапуск при входе
snowfall --autostart-off
snowfall --quit             # остановить запущенную программу
```

## Сборка из исходников

Нужны [Rust](https://rustup.rs), Node.js 20+ и, на Linux:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev librsvg2-dev \
  libayatana-appindicator3-dev patchelf libx11-dev libxi-dev libxtst-dev
```

```bash
npm install
npx tauri icon app-icon.png   # создаёт иконки для всех платформ
npx tauri build               # установщики появятся в src-tauri/target/release/bundle/
```

Если отправить в GitHub тег вида `v0.1.2`, GitHub Actions соберёт и опубликует все установщики (см. `.github/workflows/release.yml`).

## Известные ограничения

- Покрывается только основной монитор.
- **Linux:** нужен X11 или XWayland (Snowfall сам выставляет `GDK_BACKEND=x11`). В чистом Wayland-сеансе курсор и окна видны только для X11-приложений. GNOME по умолчанию не показывает значки в трее: установите расширение *AppIndicator* или останавливайте командой `snowfall --quit`.
- **macOS:** система может не пускать оверлей на полосу меню.
- Потребление памяти определяется системным WebView (обычно несколько десятков МБ).

## Лицензия

[MIT](LICENSE)
