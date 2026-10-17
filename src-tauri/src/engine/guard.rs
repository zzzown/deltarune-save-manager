//! 进程守卫、完整性指纹、原子写入

use sha2::{Digest, Sha256};
use std::path::Path;

/// 检测 DELTARUNE 游戏进程是否正在运行（Windows）
pub fn is_game_running() -> bool {
    let out = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq DELTARUNE.exe", "/FO", "CSV", "/NH"])
        .output();
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.to_lowercase().contains("deltarune")
        }
        Err(_) => false,
    }
}

#[allow(dead_code)]
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 完整性指纹：sha256 + 行数 + 首 16 行 + 末 3 行
#[allow(dead_code)]
pub fn fingerprint(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex(&hasher.finalize());
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len();
    let head = lines
        .iter()
        .take(16)
        .cloned()
        .collect::<Vec<_>>()
        .join("|");
    let tail = lines
        .iter()
        .rev()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("|");
    Ok(format!("{hash}:{line_count}:{head}:{tail}"))
}

/// 原子写入：临时文件 + 回读校验 + 备份替换
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).map_err(|e| format!("写入临时文件失败: {e}"))?;

    let back = std::fs::read(&tmp).map_err(|e| e.to_string())?;
    if back != data {
        let _ = std::fs::remove_file(&tmp);
        return Err("写入回读校验失败".into());
    }

    // Windows 上 rename 无法覆盖已存在文件，先做备份
    if path.exists() {
        let bak = path.with_extension("bak");
        let _ = std::fs::remove_file(&bak);
        std::fs::rename(path, &bak).map_err(|e| format!("备份原文件失败: {e}"))?;
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        // 回滚
        let bak = path.with_extension("bak");
        let _ = std::fs::rename(&bak, path);
        return Err(format!("替换文件失败: {e}"));
    }
    Ok(())
}
