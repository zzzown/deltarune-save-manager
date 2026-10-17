//! 快照：创建、恢复、列表、删除（整档 = 槽位×面的全章节文件 + dr.ini 相关段）

use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::types::{Face, SnapshotMeta};

/// 快照库根目录：优先读取设置中的自定义目录，否则 %APPDATA%\DeltaruneSaveManager\snapshots
fn snapshot_root() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "无法获取 APPDATA 目录".to_string())?;
    let base = PathBuf::from(&appdata).join("DeltaruneSaveManager");
    if let Ok(bytes) = fs::read(base.join("settings.json")) {
        if let Ok(s) = serde_json::from_slice::<crate::engine::types::Settings>(&bytes) {
            if !s.snapshot_dir.is_empty() {
                return Ok(PathBuf::from(s.snapshot_dir));
            }
        }
    }
    Ok(base.join("snapshots"))
}

/// 收集该槽位×面的相关文件名（常规槽 slot + completion 槽 slot+3）
fn collect_slot_filenames(save_dir: &Path, slot: u32, face: Face) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(save_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some((_ch, s, f)) = crate::engine::scanner::parse_filename(&name) {
                if f == face && (s == slot || s == slot + 3) {
                    names.push(name);
                }
            }
        }
    }
    names.sort();
    names
}

/// 提取 dr.ini 中该槽位相关的段（G_{ch}_{slot} 与 G_{ch}_{slot+3}）
fn extract_dr_sections(text: &str, slot: u32) -> String {
    let slots = [slot, slot + 3];
    let mut result = String::new();
    let mut in_target = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            let sec = &t[1..t.len() - 1];
            in_target = sec.starts_with("G_")
                && sec[2..]
                    .split('_')
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .map(|s| slots.contains(&s))
                    .unwrap_or(false);
        }
        if in_target {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

/// 解析 INI 文本为有序 (段名, 段体) 列表
fn parse_sections(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut cur_name = String::new();
    let mut cur_body = String::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            if !cur_name.is_empty() {
                out.push((cur_name, cur_body));
            }
            cur_name = t[1..t.len() - 1].to_string();
            cur_body = String::new();
        } else if !cur_name.is_empty() {
            cur_body.push_str(line);
            cur_body.push('\n');
        }
    }
    if !cur_name.is_empty() {
        out.push((cur_name, cur_body));
    }
    out
}

fn render_sections(sections: &[(String, String)]) -> String {
    let mut s = String::new();
    for (name, body) in sections {
        s.push_str(&format!("[{name}]\n{body}"));
    }
    s
}

/// 将 incoming 中的段合并进 current（同名替换，新段追加）
fn merge_dr_sections(current: &str, incoming: &str) -> String {
    let mut cur = parse_sections(current);
    let inc = parse_sections(incoming);

    for (name, body) in cur.iter_mut() {
        if let Some((_, nb)) = inc.iter().find(|(n, _)| n.as_str() == name.as_str()) {
            *body = nb.clone();
        }
    }
    for (name, body) in inc.iter() {
        if !cur.iter().any(|(n, _)| n == name) {
            cur.push((name.clone(), body.clone()));
        }
    }
    render_sections(&cur)
}

fn gen_id(slot: u32, face: Face) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let f = match face {
        Face::A => 'a',
        Face::B => 'b',
    };
    format!("{nanos}-s{slot}{f}")
}

/// 创建快照（内部实现，恢复前也会调用以备份当前状态）
fn do_create_snapshot(
    save_dir: &Path,
    slot: u32,
    face: Face,
    note: &str,
) -> Result<SnapshotMeta, String> {
    let root = snapshot_root()?;
    let id = gen_id(slot, face);
    let snap_dir = root.join(&id);
    let files_dir = snap_dir.join("files");
    fs::create_dir_all(&files_dir).map_err(|e| format!("创建快照目录失败: {e}"))?;

    let names = collect_slot_filenames(save_dir, slot, face);
    let mut total_size = 0u64;

    for name in &names {
        let src = save_dir.join(name);
        let meta = fs::metadata(&src).map_err(|e| format!("读取 {name} 失败: {e}"))?;
        total_size += meta.len();
        fs::copy(&src, files_dir.join(name))
            .map_err(|e| format!("复制 {name} 失败: {e}"))?;
    }

    // 保存 dr.ini 相关段
    if let Ok(bytes) = fs::read(save_dir.join("dr.ini")) {
        let text = String::from_utf8_lossy(&bytes).to_string();
        let sections = extract_dr_sections(&text, slot);
        fs::write(snap_dir.join("drini_sections.txt"), sections)
            .map_err(|e| format!("保存 dr.ini 段失败: {e}"))?;
    }

    let created_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
    let meta = SnapshotMeta {
        id: id.clone(),
        created_at,
        slot,
        face,
        note: note.to_string(),
        file_count: names.len(),
        total_size,
    };
    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(snap_dir.join("meta.json"), json).map_err(|e| format!("写 meta.json 失败: {e}"))?;

    Ok(meta)
}

/// 创建一个档位的快照
pub fn create_snapshot(
    save_dir: &Path,
    slot: u32,
    face: Face,
    note: &str,
) -> Result<SnapshotMeta, String> {
    do_create_snapshot(save_dir, slot, face, note)
}

/// 列出所有快照（按时间倒序）
pub fn list_snapshots() -> Result<Vec<SnapshotMeta>, String> {
    let root = snapshot_root()?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for e in fs::read_dir(&root).map_err(|e| e.to_string())?.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        if let Ok(bytes) = fs::read(e.path().join("meta.json")) {
            if let Ok(meta) = serde_json::from_slice::<SnapshotMeta>(&bytes) {
                result.push(meta);
            }
        }
    }
    result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(result)
}

/// 删除一个快照
pub fn delete_snapshot(id: &str) -> Result<(), String> {
    let root = snapshot_root()?;
    let dir = root.join(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("删除快照失败: {e}"))?;
    }
    Ok(())
}

/// 恢复快照（事务式：先备份现有文件到 _trash，再落快照文件；失败回滚）
pub fn restore_snapshot(save_dir: &Path, id: &str) -> Result<(), String> {
    // 游戏运行中禁止恢复
    if crate::engine::guard::is_game_running() {
        return Err("游戏正在运行，已禁止写入存档。请先退出游戏。".to_string());
    }

    let root = snapshot_root()?;
    let snap_dir = root.join(id);
    if !snap_dir.exists() {
        return Err(format!("快照 {id} 不存在"));
    }

    let meta: SnapshotMeta = serde_json::from_slice(
        &fs::read(snap_dir.join("meta.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("读取快照元数据失败: {e}"))?;

    // 1) 恢复前自动快照当前状态
    let _ = do_create_snapshot(save_dir, meta.slot, meta.face, "（恢复前自动备份）");

    // 2) 收集快照内的文件
    let files_dir = snap_dir.join("files");
    let mut snap_files: Vec<String> = Vec::new();
    for e in fs::read_dir(&files_dir).map_err(|e| e.to_string())?.flatten() {
        snap_files.push(e.file_name().to_string_lossy().to_string());
    }

    // 3) 事务：把目标现有文件移入 _trash
    let ts = chrono::Local::now().format("%Y%m%d%H%M%S%.3f").to_string();
    let trash_dir = root.join("_trash").join(&ts);
    fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;

    let mut moved: Vec<String> = Vec::new();
    for name in &snap_files {
        let dst = save_dir.join(name);
        if dst.exists() {
            fs::rename(&dst, trash_dir.join(name)).map_err(|e| {
                // 回滚已移动的
                for m in &moved {
                    let _ = fs::rename(trash_dir.join(m), save_dir.join(m));
                }
                format!("移动 {name} 到回收区失败: {e}")
            })?;
            moved.push(name.clone());
        }
    }

    // 4) 复制快照文件回存档目录
    for name in &snap_files {
        let src = files_dir.join(name);
        if let Err(e) = fs::copy(&src, save_dir.join(name)) {
            // 回滚：恢复已移动文件 + 删除已复制的
            for m in &moved {
                let _ = fs::rename(trash_dir.join(m), save_dir.join(m));
            }
            for name2 in &snap_files {
                if save_dir.join(name2).exists() && !moved.contains(name2) {
                    let _ = fs::remove_file(save_dir.join(name2));
                }
            }
            return Err(format!("复制 {name} 回存档目录失败: {e}"));
        }
    }

    // 5) 合并 dr.ini 段
    if let Ok(bytes) = fs::read(snap_dir.join("drini_sections.txt")) {
        let incoming = String::from_utf8_lossy(&bytes).to_string();
        let dr_path = save_dir.join("dr.ini");
        if let Ok(cur_bytes) = fs::read(&dr_path) {
            let current = String::from_utf8_lossy(&cur_bytes).to_string();
            let merged = merge_dr_sections(&current, &incoming);
            if let Err(e) = crate::engine::guard::atomic_write(&dr_path, merged.as_bytes()) {
                return Err(format!("合并 dr.ini 失败: {e}"));
            }
        }
    }

    // 6) 清理回收区（成功提交）
    let _ = fs::remove_dir_all(&trash_dir);

    Ok(())
}
