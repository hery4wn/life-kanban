# Life Kanban

A small, local Windows Kanban board built with Tauri 2, Vue 3, TypeScript, and Vite. No account, server, telemetry, remote fonts, or runtime cloud dependency. New boards start empty.

## Run it

Version 0.2 implements the reviewed sticky-note design, separate calendar and quick checklist, frog mascot, local clock, and adjustable timer. The optimized Windows installer was built successfully on GitHub Actions on 10 October 2026. See [INSTALL.md](INSTALL.md) for setup and [VALIDATION.md](VALIDATION.md) for checks and platform limits.

The setup file is `artifacts/installer-0.2/Life Kanban_0.2.0_x64-setup.exe` in this local workspace. It is also available as the **Life-Kanban-Windows-installer** artifact on [the successful private build](https://github.com/hery4wn/life-kanban/actions/runs/38018393728). The GitHub artifact is retained until 24 October 2026; the downloaded local copy remains available. The installer is unsigned. The previous 0.1 debug ZIP does not contain this redesign.

On a Windows build machine that permits the development tools, build the installer with:

```powershell
npm ci
npm run tauri -- build --bundles nsis
```

The installer is written to `src-tauri/target/release/bundle/nsis/`. It installs for the current Windows user and creates a Start menu entry. The included manually triggered GitHub Actions workflow performs the same build on a Windows runner. The user approved uploading application source to the private [hery4wn/life-kanban](https://github.com/hery4wn/life-kanban) repository. Board data and local build tools were excluded.

Run the setup executable, open Life Kanban from Start, and press Ctrl + Shift + K. Keep startup off until the installed application is in place.

The standalone executable is built at `src-tauri/target/release/life-kanban.exe` (or `target/debug/life-kanban.exe` for a debug build). Windows 10/11 x64 and the Microsoft Edge WebView2 Runtime are required. WebView2 is normally already installed; on an offline machine, install Microsoft's offline Runtime package first. No Node or Rust is needed to run a built executable.

Launch Life Kanban, then press **Ctrl + Shift + K** or use its green three-column tray icon → **Edit Kanban**. The app starts in transparent **View Mode**. The tray may be in Windows' hidden-icons overflow.

- **Add:** a column's + button → type a title → Enter. Keep typing to add another card.
- **Edit:** the arrow button on a card, double-click a note, or focus it and press Enter.
- **Move:** drag a card between columns or within a column. With a card focused, Alt + arrow keys move it using the keyboard. The card editor also has a column selector.
- **Delete:** a card's Delete button or Delete on a focused card. A confirmation defaults to keeping the card.
- **Rename a column:** click its heading in Edit Mode.
- **Finish:** Finish editing, Ctrl + Shift + K, or Escape. Escape inside a dialog closes that dialog first; Escape inside quick entry closes the entry first.
- **Quit:** tray → Quit. Closing with Alt + F4 returns to View Mode. Reopening the executable opens the existing board for editing rather than starting another instance.

Project notes show only the title. Older descriptions and tags are preserved in local JSON during migration but are hidden in the simple interface. Quick tasks are independent of project cards. The small calendar highlights today and supports month navigation; it does not store events.

The round clock uses the device's local time. Set timer minutes and seconds, then Start; Pause/Resume and Reset are available in Edit Mode. Timer configuration, pause state, and an active deadline are saved. A running timer continues across reloads and reports completion when the app is open again. Completion is a visible message; no notification or sound is configured. The clock updates once per minute while idle; the countdown updates once per second only while running.

Settings include startup (off by default), note opacity, surface blur, light/dark/system appearance, and the global shortcut. To change the shortcut, focus its field and press Ctrl and/or Alt with a letter or number, optionally with Shift. Conflicts preserve the old shortcut. Startup uses the executable's current path; keep the executable in a permanent folder before enabling it.

## Development

Install Node.js 22.12+ (or a newer supported LTS), stable Rust with the MSVC target, Visual Studio C++ Build Tools with the Windows SDK, and WebView2. Then from this folder:

```powershell
npm ci
npm run desktop:dev
```

This workspace also has `scripts/with-build-tools.ps1`. It adds Rust to the current process PATH and uses the optional project-local MSVC/SDK under `tools/msvc` if present:

```powershell
.\scripts\with-build-tools.ps1 dev
.\scripts\with-build-tools.ps1 build
.\scripts\with-build-tools.ps1 debug-build
```

The helper does not change your global PATH or security settings. The portable compiler was obtained from Microsoft's package manifests using the [portable-msvc downloader](https://gist.github.com/mmozeiko/7f3162ec2988e81e56d5c4e22cde9977). `tools/` is a local build dependency, not app source or a runtime requirement.

To build a release executable using a standard developer environment:

```powershell
npm run desktop:build
```

To create an optional NSIS installer:

```powershell
npm run tauri -- build --bundles nsis
```

Builds download dependencies the first time. Normal application use does not need a network connection. A local Windows policy can block compiler programs or generated compiler DLLs; such a block must be resolved through the device administrator's approved development setup. This project does not disable those protections.

`npm run dev` is a browser-only preview. It uses separate browser localStorage, explicitly labels itself as a preview, and disables Windows-only settings. It does not provide desktop integration.

## Your data

Desktop data lives in:

```text
%APPDATA%\com.lifekanban.desktop\board.json
```

The JSON contains column names, ordered cards, timestamps, quick checklist, timer state, and appearance/shortcut settings. The startup flag is read from Windows' startup registration rather than duplicated in the JSON.

Each completed change automatically queues a save. Rust validates the data, writes and flushes a temporary file, and replaces the previous file with a same-volume Windows rename. `board.json.bak` retains the previous valid save. If the main file is damaged, a valid backup is loaded and the damaged original is preserved under a timestamped name. If neither file can be read, the app reports the error and does not overwrite the original. Failed saves remain visible with a Retry save action; Finish editing and tray Quit wait for queued saves.

To back up your board, quit the app and copy the JSON files. To restore, quit and replace `board.json` with a known good copy. Local data is not encrypted. WebView2 keeps its own runtime cache separately under the application's local AppData directory.

## Windows desktop behavior

This version uses a **stable background-window fallback**, not true wallpaper embedding:

- A transparent, borderless WebView fills the primary monitor's work area, excluding the taskbar.
- View Mode ignores mouse events, disables focus, skips the taskbar, and uses Tauri's always-on-bottom window behavior. It makes no repeating focus or z-order requests.
- Edit Mode enables input, releases the bottom placement, and brings the board forward only on your request. A soft, nearly opaque editing backdrop keeps other open apps from distracting you.
- Settings and tray actions share the same Rust mode state. The shortcut works independently of frontend focus. Native errors are surfaced in the board.
- Layout uses logical CSS pixels; Windows work-area placement uses physical pixels. The work area is rechecked on entering Edit Mode and on DPI-change events.
- There are no polling loops or continuous animations. Reduced motion disables transitions and drag animation.

**Limits:** Windows Show desktop/Win+D, virtual desktop changes, fullscreen programs, or Explorer changes can cover/minimize a normal background window. Use the shortcut or tray to bring it back. The app does not guarantee the under-desktop-icons layer or visibility through normal app windows. It currently displays one board on one monitor; monitor selection and simultaneous multi-monitor boards are outside this first version. Scroll long boards in Edit Mode.

WorkerW embedding relies on Explorer's internal window hierarchy and an undocumented message used to create a wallpaper host. Reparenting WebView2 into another process also complicates DPI behavior and recovery after Explorer restarts. Microsoft documents [cross-process SetParent DPI risks](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent). The fallback favors documented window behavior; native placement is isolated in `src-tauri/src/desktop.rs` for future improvement. Tauri/Tao uses [SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) for bottom positioning.

**Surface blur** applies to composited layers inside the WebView. It does not blur the Windows wallpaper behind the HWND. This avoids full-window Acrylic effects that would obscure the wallpaper and can be expensive on older Windows systems.

## Structure and checks

```text
src/components/     notes, columns, dialogs
src/views/          board composition
src/composables/    native mode/events bridge
src/stores/         ordered board mutations and save queue
src/types/          data shapes
src/utils/          validation, moves, desktop/browser persistence
src-tauri/src/      local JSON, tray, hotkey, startup, Windows placement
tests/             data and browser interaction checks
```

```powershell
npm run build
npm test
npm run test:ui
.\scripts\with-build-tools.ps1 test
```

Browser tests use installed Microsoft Edge, isolate their data, and exercise pointer dragging, ordering, keyboard moves, CRUD, confirmation, themes, reload persistence, damaged-data handling, and desktop viewport sizes. Rust tests exercise actual filesystem writes, backup recovery, and data validation. See `VALIDATION.md` for checks performed on this machine and remaining platform coverage.
