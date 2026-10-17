//! 与前端一一对应的共享类型（serde 序列化）

use serde::{Deserialize, Serialize};

/// 存档面：A = 正篇，B = 怪异路线（Side B）
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Face {
    A,
    B,
}

impl Face {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Face::A => "正篇",
            Face::B => "B 面",
        }
    }
}

/// 槽位元数据（来自 dr.ini 的 [G_{ch}_{slot}] 段）
#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SlotMeta {
    pub chapter: u32,
    pub slot: u32,
    pub name: String,
    pub level: f64,
    pub love: f64,
    pub time: f64,
    pub date: f64,
    pub room: f64,
    pub side_b: f64,
    pub ura_boss: f64,
    pub version: String,
    pub empty: bool,
}

/// 单个存档文件条目
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub filename: String,
    pub chapter: u32,
    pub slot: u32,
    pub face: Face,
    pub path: String,
    pub size: u64,
    pub line_count: usize,
}

/// 健康状态
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Ok,
    Warn,
    #[allow(dead_code)]
    Broken,
}

/// 一个存档档位（槽位 × 面）
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub slot: u32,
    pub face: Face,
    pub chapters: Vec<u32>,
    pub name: String,
    pub level: f64,
    pub time: f64,
    pub side_b: bool,
    pub completed_chapters: Vec<u32>,
    pub health: Health,
    pub issues: Vec<String>,
}

/// 扫描结果
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub save_dir: String,
    pub profiles: Vec<Profile>,
    pub files: Vec<SaveFile>,
    pub other_files: Vec<String>,
    pub game_running: bool,
}

/// 快照元数据
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub id: String,
    pub created_at: String,
    pub slot: u32,
    pub face: Face,
    pub note: String,
    pub file_count: usize,
    pub total_size: u64,
    /// 星标锁定：不占保留配额、不被自动清理
    pub locked: bool,
}

/// 应用设置
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub game_path: String,
    pub snapshot_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            game_path: String::new(),
            snapshot_dir: String::new(),
        }
    }
}
