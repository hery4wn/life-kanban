# Validation — 10 October 2026

## Version 0.2 integration

The approved interface is implemented in the Vue application: title-only sticky notes, friendlier fonts and column icons, frog character, desktop icon clearance, separate calendar and quick checklist, and a round clock with configurable timer below the checklist. No sample cards or tasks are added to real boards.

TypeScript checking and production frontend compilation passed. Six unit tests and eleven Microsoft Edge UI tests passed, including legacy JSON migration, quick-task persistence, calendar navigation, dragging/reordering, confirmation, appearance settings, and timer pause/resume/reload/completion/reset. A final visual review used the built frontend at 1366×768. Dependency audit reported zero vulnerabilities.

**Installer blocked:** on this date Windows Application Control denied execution of Cargo and Rust, including the directly installed toolchain binaries, with OS error 4551. The NSIS installer build stopped before Rust compilation. New native migration tests are present but could not be executed. Version 0.2 native compilation, install/uninstall, and target Windows acceptance remain unverified. No security policy was changed.

The `.github/workflows/windows-installer.yml` workflow is prepared for an approved Windows runner. It has not been uploaded or executed. `artifacts/Life-Kanban-0.2-source.zip` contains the source and build instructions, **not a compiled app or installer**. The older ZIP described below remains version 0.1 and does not contain the redesign.

## Delivered build

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
