//! DELTARUNE 外置存档管理器 —— Tauri 应用入口与命令注册

pub mod engine;

use std::path::PathBuf;

use crate::engine::types::{Face, Settings, SnapshotMeta, ScanResult};

/// 默认存档目录
fn save_dir() -> Result<PathBuf, String> {
    crate::engine::scanner::default_save_dir().ok_or_else(|| "未找到 DELTARUNE 存档目录（%LOCALAPPDATA%\\DELTARUNE）".to_string())
}

/// 扫描存档目录
#[tauri::command]
fn scan() -> Result<ScanResult, String> {
    crate::engine::scanner::scan(&save_dir()?)
}

/// 列出所有快照
#[tauri::command]
fn list_snapshots() -> Result<Vec<SnapshotMeta>, String> {
    crate::engine::snapshot::list_snapshots()
}

/// 为一整个档位（槽位×面）创建快照
#[tauri::command]
fn create_snapshot(slot: u32, face: Face, note: String) -> Result<SnapshotMeta, String> {
    crate::engine::snapshot::create_snapshot(&save_dir()?, slot, face, &note)
}

/// 恢复快照
#[tauri::command]
fn restore_snapshot(id: String) -> Result<(), String> {
    crate::engine::snapshot::restore_snapshot(&save_dir()?, &id)
}

/// 删除快照（软删除，移入回收区）
#[tauri::command]
fn delete_snapshot(id: String) -> Result<(), String> {
    crate::engine::snapshot::delete_snapshot(&id)
}

/// 启动游戏
#[tauri::command]
fn launch_game() -> Result<(), String> {
    let settings = load_settings();
    let path = if !settings.game_path.is_empty() {
        settings.game_path
    } else {
        detect_game_path().ok_or_else(|| {
            "未配置游戏路径，请在设置中填写 DELTARUNE.exe 的完整路径。".to_string()
        })?
    };
    // spawn 返回的 Child 被 drop 时不会终止进程，游戏会继续运行
    let _child = std::process::Command::new(&path)
        .spawn()
        .map_err(|e| format!("启动失败: {e}"))?;
    Ok(())
}

/// 读取设置
#[tauri::command]
fn get_settings() -> Settings {
    load_settings()
}

/// 保存设置
#[tauri::command]
fn set_settings(settings: Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

fn settings_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    PathBuf::from(appdata)
        .join("DeltaruneSaveManager")
        .join("settings.json")
}

fn load_settings() -> Settings {
    if let Ok(bytes) = std::fs::read(settings_path()) {
        if let Ok(s) = serde_json::from_slice::<Settings>(&bytes) {
            return s;
        }
    }
    Settings::default()
}

/// 探测常见 Steam 安装路径下的 DELTARUNE.exe
fn detect_game_path() -> Option<String> {
    const CANDIDATES: [&str; 6] = [
        r"C:\Program Files (x86)\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe",
        r"C:\Program Files\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe",
        r"D:\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe",
        r"D:\SteamLibrary\steamapps\common\DELTARUNE\DELTARUNE.exe",
        r"E:\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe",
        r"E:\SteamLibrary\steamapps\common\DELTARUNE\DELTARUNE.exe",
    ];
    CANDIDATES
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|s| s.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            scan,
            list_snapshots,
            create_snapshot,
            restore_snapshot,
            delete_snapshot,
            launch_game,
            get_settings,
            set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("运行 DELTARUNE 存档管理器时出错");
}
