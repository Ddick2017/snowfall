# Snowfall ❄

Лёгкий оверлей со снегом для рабочего стола (аналог Flake для macOS), для macOS, Linux и Windows 10/11.

- снег падает поверх всех окон и **ложится на верхнюю кромку окон и на панель задач / Dock**
- снег можно **смахнуть мышкой** (клики при этом проходят сквозь оверлей, окна работают как обычно)
- в **полноэкранном режиме** (видео, игры, F11) снег полностью останавливается
- **нет значка** в Dock / на панели задач и нет окна; один раз включаешь автозапуск и забываешь
- стек: Rust + Tauri (системный WebView), ~140 снежинок на canvas, 30 fps, опрос окон ~5 раз в секунду

## Автозапуск

```bash
snowfall --autostart-on     # включить автозапуск при входе в систему
snowfall --autostart-off    # выключить
snowfall --quit             # остановить запущенный снегопад
```

- **Linux .deb**: `sudo apt install ./snowfall_*.deb`, затем `snowfall --autostart-on`
- **Linux .tar.gz**: распаковать, `./install-linux.sh`
- **Linux .AppImage**: `chmod +x Snowfall*.AppImage && ./Snowfall*.AppImage --autostart-on`
- **macOS**: открыть .dmg, перетащить в «Программы», затем
  `xattr -dr com.apple.quarantine /Applications/Snowfall.app` (сборка без подписи Apple) и
  `/Applications/Snowfall.app/Contents/MacOS/snowfall --autostart-on`
- **Windows**: установщик `.exe`/`.msi` или portable `.zip`; `snowfall.exe --autostart-on` (запускать из cmd/PowerShell)

## Сборка и публикация релизов

1. `app-icon.png` — исходная иконка (можно заменить на свою, 1024×1024).
2. Залить репозиторий на GitHub:
   ```bash
   git init -b main
   git add .
   git commit -m "Snowfall: first commit"
   git remote add origin git@github.com:ВАШ_ЛОГИН/snowfall.git
   git push -u origin main
   ```
3. Выпустить релиз — GitHub Actions соберёт всё сам на Linux, macOS (Apple Silicon + Intel) и Windows:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
   Через 10–20 минут во вкладке **Releases** появятся `.deb`, `.AppImage`, `.tar.gz`, `.dmg`, `.exe`, `.msi`, portable `.zip`.

Локальная сборка: `npm install && npx tauri icon app-icon.png && npx tauri build`
(Linux: `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev librsvg2-dev patchelf`).

## Ограничения

- Работает на основном мониторе.
- Linux: нужен X11 или XWayland (программа сама выставляет `GDK_BACKEND=x11`). В чистом Wayland-сеансе положение курсора и окон видно только для X11-окон.
- Windows: нужен WebView2 (есть в Windows 11 и в обновлённой Windows 10).
- macOS: оверлей не заходит на полоску меню, если система его туда не пускает.
- Потребление ОЗУ определяется системным WebView (порядка 40–70 МБ); это не «ноль», но заметно меньше Electron.
