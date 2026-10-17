//! 存档目录扫描 + 文件名解析 + 档位（Profile）构建

use std::path::{Path, PathBuf};

use crate::engine::metadata::{parse_dr_ini, DrIni};
use crate::engine::types::{Face, Health, Profile, SaveFile, ScanResult};

/// 默认存档目录：%LOCALAPPDATA%\DELTARUNE
pub fn default_save_dir() -> Option<PathBuf> {
    let la = std::env::var("LOCALAPPDATA").ok()?;
    let p = PathBuf::from(la).join("DELTARUNE");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

/// 发现 %LOCALAPPDATA% 下所有 DELTARUNE* 目录（如 DELTARUNE、DELTARUNE_bak、DELTARUNE_pink）
pub fn discover_save_dirs() -> Vec<String> {
    let Ok(la) = std::env::var("LOCALAPPDATA") else {
        return Vec::new();
    };
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&la) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("DELTARUNE") && e.path().is_dir() {
                result.push(e.path().to_string_lossy().to_string());
            }
        }
    }
    result.sort();
    result
}

/// 解析文件名 `filech{N}_{slot}[_b]`
pub fn parse_filename(name: &str) -> Option<(u32, u32, Face)> {
    let rest = name.strip_prefix("filech")?;
    let (ch_str, slot_part) = rest.split_once('_')?;
    let chapter: u32 = ch_str.parse().ok()?;
    let (slot_str, face) = if let Some(s) = slot_part.strip_suffix("_b") {
        (s, Face::B)
    } else {
        (slot_part, Face::A)
    };
    let slot: u32 = slot_str.parse().ok()?;
    Some((chapter, slot, face))
}

fn count_lines(path: &Path) -> usize {
    let Ok(b) = std::fs::read(path) else {
        return 0;
    };
    if b.is_empty() {
        return 0;
    }
    // 逻辑行数 = 换行符数量 + 1（若最后一行无换行符）
    b.iter().filter(|&&c| c == b'\n').count() + usize::from(!b.ends_with(b"\n"))
}

/// 扫描存档目录，构建槽位×章节×面矩阵
pub fn scan(save_dir: &Path) -> Result<ScanResult, String> {
    let entries =
        std::fs::read_dir(save_dir).map_err(|e| format!("读取存档目录失败: {e}"))?;

    let mut files: Vec<SaveFile> = Vec::new();
    let mut other_files: Vec<String> = Vec::new();

    for e in entries {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().to_string();

        if let Some((chapter, slot, face)) = parse_filename(&name) {
            let meta = e.metadata().ok();
            let size = meta.map(|m| m.len()).unwrap_or(0);
            let line_count = count_lines(&e.path());
            files.push(SaveFile {
                filename: name,
                chapter,
                slot,
                face,
                path: e.path().to_string_lossy().to_string(),
                size,
                line_count,
            });
        } else if name.starts_with("filech") {
            // filech 开头但无法解析（如 filech4_2_old），只备份不解析
            other_files.push(name);
        }
    }

    let dr_ini = parse_dr_ini(&save_dir.join("dr.ini")).unwrap_or_else(|_| DrIni {
        slots: Default::default(),
        ura: Default::default(),
        f_items: Default::default(),
    });

    let profiles = build_profiles(&files, &dr_ini);

    Ok(ScanResult {
        save_dir: save_dir.to_string_lossy().to_string(),
        profiles,
        files,
        other_files,
        game_running: crate::engine::guard::is_game_running(),
    })
}

fn build_profiles(files: &[SaveFile], dr_ini: &DrIni) -> Vec<Profile> {
    let mut profiles = Vec::new();

    for slot in 0u32..3 {
        for face in [Face::A, Face::B] {
            // 常规存档章节（slot 0–2）
            let mut chapters: Vec<u32> = files
                .iter()
                .filter(|f| f.slot == slot && f.face == face)
                .map(|f| f.chapter)
                .collect();
            chapters.sort_unstable();
            chapters.dedup();

            // completion 章节（slot 3–5 对应档位 0–2）
            let comp_slot = slot + 3;
            let mut completed: Vec<u32> = files
                .iter()
                .filter(|f| f.slot == comp_slot && f.face == face)
                .map(|f| f.chapter)
                .collect();
            completed.sort_unstable();
            completed.dedup();

            if chapters.is_empty() && completed.is_empty() {
                continue; // 空档不生成
            }

            // 从 dr.ini 取该档位（常规槽 + completion 槽）的所有元数据，按章节排序
            let mut metas: Vec<&crate::engine::types::SlotMeta> = dr_ini
                .slots
                .iter()
                .filter(|((_ch, s), _)| *s == slot || *s == comp_slot)
                .map(|(_, m)| m)
                .collect();
            metas.sort_by_key(|m| m.chapter);

            // 取「有实际存档的最高章节」元数据（跳过 [EMPTY] 空槽），否则退回最高章节
            let latest = metas.iter().rev().find(|m| !m.empty).or_else(|| metas.last());
            let name = latest.map(|m| m.name.clone()).unwrap_or_default();
            let level = latest.map(|m| m.level).unwrap_or(0.0);
            let time = latest.map(|m| m.time).unwrap_or(0.0);

            // B 面判定：任一章节 SideB 标志为 1，或本身就是 B 面文件
            let side_b = metas.iter().any(|m| m.side_b >= 1.0) || face == Face::B;

            // 体检
            let mut issues: Vec<String> = Vec::new();

            // 玩家名跨章一致
            let mut names: Vec<&str> = metas
                .iter()
                .filter(|m| !m.empty)
                .map(|m| m.name.as_str())
                .collect();
            names.sort_unstable();
            names.dedup();
            if names.len() > 1 {
                issues.push("多个章节的玩家名不一致".to_string());
            }

            // 章节链断裂（常规存档）
            for w in chapters.windows(2) {
                if w[1] != w[0] + 1 {
                    issues.push(format!("章节链在 Ch{} 与 Ch{} 之间断裂", w[0], w[1]));
                    break;
                }
            }

            // B 面文件存在但 dr.ini 未标记
            if face == Face::B && !metas.iter().any(|m| m.side_b >= 1.0) {
                issues.push("存在 B 面文件，但 dr.ini 未标记 SideB".to_string());
            }

            let health = if issues.is_empty() {
                Health::Ok
            } else {
                Health::Warn
            };

            profiles.push(Profile {
                slot,
                face,
                chapters,
                name,
                level,
                time,
                side_b,
                completed_chapters: completed,
                health,
                issues,
            });
        }
    }

    profiles
}
