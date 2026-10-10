# Install Life Kanban

Life Kanban 0.2 is for Windows 10/11, 64-bit. You do not need Node.js or Rust to use it.

1. Run the `Life Kanban_0.2.0_x64-setup.exe` installer.
2. Open **Life Kanban** from the Windows Start menu.
3. Press **Ctrl + Shift + K** to edit your board. The application starts in a quiet, transparent View Mode.
4. Add project titles to the four Kanban columns. Add small tasks in the separate quick checklist. Set minutes and seconds beside the round clock to start a focus timer.
5. Click **Finish editing**, or press the shortcut again, to return to View Mode. Your changes save automatically on your computer.

The tray icon also provides **Edit Kanban**, **Open Settings**, and **Quit**. It may appear in Windows' hidden-icons menu. Enable **Start with Windows** in Settings if you want the board to launch automatically.

The installer installs for your current Windows user. It includes the application and can install Microsoft's WebView2 Runtime if it is missing; that first-time download needs an internet connection. Normal board use works offline.

The application is not digitally signed. If Windows blocks it, do not change your computer's security settings; use an administrator-approved installation route.

## Keep your board safe

Data is stored in `%APPDATA%\com.lifekanban.desktop\board.json`. A previous valid save is kept in `board.json.bak`. Quit the application before copying these files as a backup. Updating the application keeps its existing data folder.

You can uninstall Life Kanban from **Settings → Apps → Installed apps**. Keep a backup of your board first.

See `README.md` for keyboard controls and desktop behavior, and `VALIDATION.md` for the checks performed and remaining Windows acceptance checks.
