# Validation — 10 October 2026

## Version 0.2.1 visibility and startup repair

The root background reset now overrides the preview's `:root` background. View Mode leaves the desktop visible; opacity affects panel backgrounds instead of fading text and controls. The app title has a contrasting surface and a visible version number.

Native Edit/View changes now run in order on the Windows UI thread. Previously, Tauri could queue the click-through style change while Win32 alpha initialization ran immediately on another thread, leaving the newly layered window uninitialized and invisible. The release app also refreshes an existing enabled startup entry to its current executable without enabling startup for users who had it off.

Frontend compilation, six unit tests, and twelve Edge interaction tests passed. The added regression check verifies transparent root and View Mode backgrounds, readable panel contents at 45% opacity, and the visible version label. The private Windows build 38025265238 passed all three native persistence tests, optimized compilation, and installer packaging. Version 0.2.1 was installed over 0.2.0 on this computer; the installer returned exit code 0 and Windows reports version 0.2.1. The installed app shows its version, opens Edit Mode using Ctrl+Shift+K, and returns to View Mode using Escape. The user confirmed that the board is visible on the desktop with other windows minimized. Window diagnostics report visible=true and layered alpha=255. Existing 13 project cards and one quick task are unchanged against the pre-install backup. The old portable process was closed and Windows startup now points to the installed executable rather than the development copy. A physical reboot was not performed; tray, click-through, DPI and extended idle acceptance remain separate checks. Installer SHA-256: 8A0EE6467DDB236E211B3BEA6AB1145D9FC0804B97F970FF893DB9D161D84A30. Downloaded archive SHA-256 matched GitHub artifact digest 4cb29dfb9aa82d0f88b84e69d35455c3f27f0670c1d204a53bf58ad52b0e94f0. The native screenshot capture of this click-through background window can show the foreground application instead of the board; it is not proof that the board is hidden.

## Version 0.2 integration

The approved interface is implemented in the Vue application: title-only sticky notes, friendlier fonts and column icons, frog character, desktop icon clearance, separate calendar and quick checklist, and a round clock with configurable timer below the checklist. No sample cards or tasks are added to real boards.

TypeScript checking and production frontend compilation passed. Six unit tests and eleven Microsoft Edge UI tests passed, including legacy JSON migration, quick-task persistence, calendar navigation, dragging/reordering, confirmation, appearance settings, and timer pause/resume/reload/completion/reset. A final visual review used the built frontend at 1366×768. Dependency audit reported zero vulnerabilities.

**Local build restriction:** on this date Windows Application Control denied execution of Cargo and Rust, including the directly installed toolchain binaries, with OS error 4551. The local NSIS installer build stopped before Rust compilation. No security policy was changed.

The user authorized a private GitHub repository and a Windows Actions build. The source is at [hery4wn/life-kanban](https://github.com/hery4wn/life-kanban), with no local board data, attachments, compiler tools, or dependency folders uploaded. [Build run 38018393728](https://github.com/hery4wn/life-kanban/actions/runs/38018393728) completed successfully in 11 minutes 10 seconds, passing frontend compilation, six unit tests, all three native persistence tests, optimized native compilation, and NSIS installer packaging. The build used application commit `f490307113e9b8304470b2a8e4266edb4bd1a18f`.

The installer was downloaded and extracted to `artifacts/installer-0.2/Life Kanban_0.2.0_x64-setup.exe` (1,241,811 bytes). Its Windows executable headers were checked. SHA-256: `0A78CE7D1FC16F743CF2B044B892269F5E08DB7C030FDFFA3673EF64D80B895B`. The downloaded ZIP matched GitHub's artifact digest `661fb30b8ab07a36db2bf030565ce7176afe93215d7d5f6502491267949b3710`. The installer is not digitally signed.

Version 0.2 install/uninstall and interactive native tray, startup, and click-through acceptance on the target computer remain unverified. Automated checks and successful packaging do not establish those behaviors. The older manual checks below apply only to version 0.1.

`artifacts/Life-Kanban-0.2-source.zip` contains source and build instructions, **not a compiled app or installer**. The older ZIP described below remains version 0.1 and does not contain the redesign.

## Historical version 0.1 test build (4 October 2026)

`artifacts/Life-Kanban-Windows-x64.zip` contains a standalone Windows x64 **debug/test build**, usage instructions, and this report. Requires Microsoft Edge WebView2 Runtime. No Node.js or Rust is needed to run it. It is not a signed installer or optimized production release.

The final Tauri debug build succeeded, including Vue/TypeScript checking and production frontend asset compilation. Windows Application Control blocked a generated `windows_interface` compiler DLL during the optimized Rust release build (Code Integrity event 3077). It also blocked Clippy. No Windows security settings were changed. An approved Windows build environment is needed to produce and validate the optimized release and installer.

## Passed checks

- 4 unit tests: within-column ordering, cross-column movement, invalid destinations, malformed/duplicate data.
- 8 Microsoft Edge UI tests: card creation/editing/deletion with confirmation, column rename, real pointer dragging and reordering, reload persistence, keyboard moves, Edit/View transitions, appearance settings, damaged-data preservation, and layouts at 1366×768, 1920×1080, 2560×1440, and 911×512.
- 2 Rust tests: real filesystem save/backup/recovery and preservation of corrupt originals; native validation of duplicates and invalid settings.
- Native Windows manual checks on an isolated test board: launch, Ctrl+Shift+K entering Edit Mode, Escape returning to View Mode, quick card creation saved to JSON, pointer dragging across columns saved to JSON, and opening Settings with startup off.
- Dependency audit after updating the test runner reported zero vulnerabilities.

A final browser test run initially exposed a file-watcher conflict while Rust was building. Vite now excludes native build output and local compiler files; all eight browser tests then passed.

## Remaining platform acceptance

Tray menu actions, startup registration/reboot, shortcut conflict recovery, click-through to desktop icons, multi-monitor/DPI changes, and extended idle resource usage still need direct end-to-end acceptance on the target Windows installation. Browser resolution tests are not proof of native DPI behavior. Windows 10 was not separately tested.

The app deliberately uses a normal background window rather than undocumented WorkerW embedding. Windows Show desktop, virtual desktop changes, or Explorer behavior may hide it; the shortcut restores Edit Mode. Surface blur does not blur the Windows wallpaper. See README for the implementation and limits.

## Repeating the checks

```powershell
npm ci
npm run build
npm test
npm run test:ui
.\scripts\with-build-tools.ps1 test
.\scripts\with-build-tools.ps1 debug-build
```

Native test data used a debug-only `LIFE_KANBAN_TEST_DATA_DIR` override, separate from the user's real AppData board. The delivered app starts with an empty board when no saved board exists.

