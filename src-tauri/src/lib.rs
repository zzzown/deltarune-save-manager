//! DELTARUNE 外置存档管理器 —— Tauri 应用入口与命令注册

pub mod engine;

use std::path::PathBuf;

use crate::engine::types::{Face, Settings, SnapshotMeta, ScanResult};

/// 默认存档目录
fn default_save_dir() -> Result<PathBuf, String> {
    crate::engine::scanner::default_save_dir().ok_or_else(|| "未找到 DELTARUNE 存档目录（%LOCALAPPDATA%\\DELTARUNE）".to_string())
}

/// 扫描存档目录（save_dir 为空则用默认目录）
#[tauri::command]
fn scan(save_dir: Option<String>) -> Result<ScanResult, String> {
    let dir = match save_dir.as_deref() {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => default_save_dir()?,
    };
    crate::engine::scanner::scan(&dir)
}

/// 发现所有 DELTARUNE 存档目录
#[tauri::command]
fn list_save_dirs() -> Vec<String> {
    crate::engine::scanner::discover_save_dirs()
}

/// 列出所有快照
#[tauri::command]
fn list_snapshots() -> Result<Vec<SnapshotMeta>, String> {
    crate::engine::snapshot::list_snapshots()
}

/// 为一整个档位（槽位×面）创建快照
#[tauri::command]
fn create_snapshot(slot: u32, face: Face, note: String) -> Result<SnapshotMeta, String> {
    crate::engine::snapshot::create_snapshot(&default_save_dir()?, slot, face, &note)
}

/// 恢复快照
#[tauri::command]
fn restore_snapshot(id: String) -> Result<(), String> {
    crate::engine::snapshot::restore_snapshot(&default_save_dir()?, &id)
}

/// 删除快照（软删除，移入回收区）
#[tauri::command]
fn delete_snapshot(id: String) -> Result<(), String> {
    crate::engine::snapshot::delete_snapshot(&id)
}

/// 切换快照星标锁定
#[tauri::command]
fn toggle_lock(id: String, locked: bool) -> Result<SnapshotMeta, String> {
    crate::engine::snapshot::toggle_lock(&id, locked)
}

/// 列出回收站快照
#[tauri::command]
fn list_trash() -> Result<Vec<SnapshotMeta>, String> {
    crate::engine::snapshot::list_trash()
}

/// 从回收站恢复快照
#[tauri::command]
fn restore_trash(id: String) -> Result<(), String> {
    crate::engine::snapshot::restore_trash(&id)
}

/// 彻底删除回收站快照
#[tauri::command]
fn purge_trash(id: String) -> Result<(), String> {
    crate::engine::snapshot::purge_trash(&id)
}

/// 启动游戏
#[tauri::command]
fn launch_game() -> Result<(), String> {
    let settings = load_settings();
    if settings.launch_method == "steam" {
        // 通过 Steam 启动（AppID 1671210 为 Deltarune 正式版）
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", "steam://rungameid/1671210"])
            .spawn()
            .map_err(|e| format!("通过 Steam 启动失败: {e}"))?;
        return Ok(());
    }
    let path = if !settings.game_path.is_empty() {
        settings.game_path
    } else {
        find_game_path().ok_or_else(|| {
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

/// 自动探测游戏 exe 路径（暴露给前端）
#[tauri::command]
fn detect_game_path() -> Option<String> {
    find_game_path()
}

/// 探测本机 DELTARUNE.exe：Steam 默认路径 → 注册表 Steam 目录 → 手动放置的常见位置
pub fn find_game_path() -> Option<String> {
    let mut candidates: Vec<String> = vec![
        r"C:\Program Files (x86)\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"C:\Program Files\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"D:\Program Files (x86)\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"D:\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"D:\SteamLibrary\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"E:\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
        r"E:\SteamLibrary\steamapps\common\DELTARUNE\DELTARUNE.exe".into(),
    ];

    if let Some(steam) = reg_steam_install_path() {
        candidates.push(format!(
            "{steam}\\steamapps\\common\\DELTARUNE\\DELTARUNE.exe"
        ));
    }

    // 手动放置的常见位置（Saved Games / Games，各盘符）
    for drive in ['C', 'D', 'E', 'F', 'G'] {
        candidates.push(format!("{drive}:\\Saved Games\\DELTARUNE\\DELTARUNE.exe"));
        candidates.push(format!("{drive}:\\Games\\DELTARUNE\\DELTARUNE.exe"));
    }

    candidates
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
}

/// 通过注册表查询 Steam 安装目录
fn reg_steam_install_path() -> Option<String> {
    let out = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\WOW6432Node\Valve\Steam",
            "/v",
            "InstallPath",
        ])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let marker = "REG_SZ";
    let idx = s.rfind(marker)?;
    let val = s[idx + marker.len()..].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan,
            list_save_dirs,
            list_snapshots,
            create_snapshot,
            restore_snapshot,
            delete_snapshot,
            toggle_lock,
            list_trash,
            restore_trash,
            purge_trash,
            launch_game,
            get_settings,
            set_settings,
            detect_game_path,
        ])
        .run(tauri::generate_context!())
        .expect("运行 DELTARUNE 存档管理器时出错");
}
