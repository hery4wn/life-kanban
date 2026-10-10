use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::Manager;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tag: String,
    pub created_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub cards: Vec<Card>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub opacity: u8,
    pub blur: u8,
    pub theme: String,
    pub shortcut: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct QuickTask {
    pub id: String,
    pub title: String,
    pub done: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub duration_seconds: u32,
    pub remaining_seconds: f64,
    pub ends_at: Option<u64>,
}
impl Default for TimerState {
    fn default() -> Self {
        Self { duration_seconds: 1500, remaining_seconds: 1500.0, ends_at: None }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Board {
    pub version: u8,
    #[serde(default, rename = "quickTasks")]
    pub quick_tasks: Vec<QuickTask>,
    #[serde(default)]
    pub timer: TimerState,
    pub columns: Vec<Column>,
    pub settings: Settings,
}
#[derive(Serialize)]
pub struct LoadResult {
    pub board: Board,
    pub warning: Option<String>,
}
pub struct DataState {
    pub path: PathBuf,
    pub lock: Mutex<()>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            version: 1,
            quick_tasks: vec![],
            timer: TimerState::default(),
            columns: ["Backlog", "To do", "In progress", "Done"]
                .iter()
                .enumerate()
                .map(|(i, name)| Column {
                    id: format!("column-{i}"),
                    name: name.to_string(),
                    cards: vec![],
                })
                .collect(),
            settings: Settings {
                opacity: 85,
                blur: 12,
                theme: "system".into(),
                shortcut: "Control+Shift+K".into(),
            },
        }
    }
}
impl Board {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.columns.len() != 4 {
            return Err("Unsupported board version or column count".into());
        }
        let mut ids = HashSet::new();
        for col in &self.columns {
            if col.id.is_empty()
                || !ids.insert(&col.id)
                || col.name.trim().is_empty()
                || col.name.chars().count() > 40
            {
                return Err("Invalid column".into());
            }
            for card in &col.cards {
                if card.id.is_empty()
                    || !ids.insert(&card.id)
                    || card.title.trim().is_empty()
                    || card.title.chars().count() > 200
                    || card.description.chars().count() > 2000
                    || card.tag.chars().count() > 40
                    || card.created_at.is_empty()
                {
                    return Err("Invalid card".into());
                }
            }
        }
        for task in &self.quick_tasks {
            if task.id.is_empty() || !ids.insert(&task.id) || task.title.trim().is_empty() || task.title.chars().count() > 120 { return Err("Invalid quick task".into()); }
        }
        if !(1..=10859).contains(&self.timer.duration_seconds) || !self.timer.remaining_seconds.is_finite() || self.timer.remaining_seconds < 0.0 || self.timer.remaining_seconds > self.timer.duration_seconds as f64 || self.timer.ends_at.is_some_and(|v| v > 9007199254740991) { return Err("Invalid timer".into()); }
        if !(35..=100).contains(&self.settings.opacity)
            || self.settings.blur > 24
            || !["system", "light", "dark"].contains(&self.settings.theme.as_str())
            || self.settings.shortcut.is_empty()
        {
            return Err("Invalid settings".into());
        }
        Ok(())
    }
}
pub fn read(path: &Path) -> Result<Board, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let board: Board = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    board.validate()?;
    Ok(board)
}
pub fn load(path: &Path) -> Result<LoadResult, String> {
    if !path.exists() {
        return Ok(LoadResult {
            board: Board::default(),
            warning: None,
        });
    }
    match read(path) {
        Ok(board) => Ok(LoadResult {
            board,
            warning: None,
        }),
        Err(original) => {
            let backup = path.with_extension("json.bak");
            match read(&backup) {
                Ok(board) => {
                    // Preserve damaged input before allowing subsequent automatic saves.
                    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
                    fs::copy(path, path.with_extension(format!("damaged-{stamp}.json"))).map_err(|e| e.to_string())?;
                    Ok(LoadResult { board, warning: Some("Recovered your previous saved board. The damaged file was preserved beside it.".into()) })
                },
                Err(_) => Err(format!("{original}. The original file was left untouched. Restore board.json from a backup in the data folder.")),
            }
        }
    }
}

fn atomic_replace(source: &Path, target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };
        let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let target: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
        // Same-volume rename replaces the old file only after the complete new file is flushed.
        if unsafe {
            MoveFileExW(
                source.as_ptr(),
                target.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        fs::rename(source, target).map_err(|e| e.to_string())
    }
}
pub fn save(path: &Path, board: &Board) -> Result<(), String> {
    board.validate()?;
    fs::create_dir_all(path.parent().ok_or("Invalid data folder")?).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    let mut file = File::create(&temp).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(board).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    // Never replace a good backup with corrupt data from a failed previous startup.
    if path.exists() && read(path).is_ok() {
        let backup_temp = path.with_extension("json.bak.tmp");
        fs::copy(path, &backup_temp).map_err(|e| e.to_string())?;
        fs::OpenOptions::new()
            .write(true)
            .open(&backup_temp)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        atomic_replace(&backup_temp, &path.with_extension("json.bak"))?;
    }
    atomic_replace(&temp, path)
}
#[tauri::command]
pub fn load_board(app: tauri::AppHandle) -> Result<LoadResult, String> {
    let state = app.state::<DataState>();
    let _lock = state.lock.lock().map_err(|e| e.to_string())?;
    load(&state.path)
}
#[tauri::command]
pub fn save_board(app: tauri::AppHandle, board: Board) -> Result<(), String> {
    let state = app.state::<DataState>();
    let _lock = state.lock.lock().map_err(|e| e.to_string())?;
    #[cfg(debug_assertions)]
    eprintln!("Saving board to {}", state.path.display());
    save(&state.path, &board)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saves_recovers_and_preserves_corruption() {
        let dir = std::env::temp_dir().join(format!(
            "life-kanban-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("board.json");
        let mut board = Board::default();
        save(&path, &board).unwrap();
        board.columns[0].name = "Ideas".into();
        save(&path, &board).unwrap();
        assert_eq!(read(&path).unwrap().columns[0].name, "Ideas");
        fs::write(&path, "{broken").unwrap();
        let recovered = load(&path).unwrap();
        assert!(recovered.warning.is_some());
        assert_eq!(recovered.board.columns[0].name, "Backlog");
        assert!(fs::read_dir(&dir).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("damaged")));
        save(&path, &recovered.board).unwrap();
        assert!(read(&path).is_ok());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn upgrades_legacy_data_and_round_trips_widgets() {
        let mut value = serde_json::to_value(Board::default()).unwrap();
        value.as_object_mut().unwrap().remove("quickTasks");
        value.as_object_mut().unwrap().remove("timer");
        let mut migrated: Board = serde_json::from_value(value).unwrap();
        assert!(migrated.quick_tasks.is_empty());
        assert_eq!(migrated.timer.duration_seconds, 1500);
        migrated.quick_tasks.push(QuickTask { id: "quick-1".into(), title: "Water plants".into(), done: true });
        migrated.timer.ends_at = Some(1791600000000);
        let restored: Board = serde_json::from_slice(&serde_json::to_vec(&migrated).unwrap()).unwrap();
        restored.validate().unwrap();
        assert!(restored.quick_tasks[0].done);
        assert_eq!(restored.timer.ends_at, migrated.timer.ends_at);
        migrated.quick_tasks[0].id = migrated.columns[0].id.clone();
        assert!(migrated.validate().is_err());
    }
    #[test]
    fn rejects_duplicate_ids_and_invalid_settings() {
        let mut board = Board::default();
        board.columns[1].id = board.columns[0].id.clone();
        assert!(board.validate().is_err());
        let mut board = Board::default();
        board.settings.opacity = 0;
        assert!(board.validate().is_err());
    }
}
