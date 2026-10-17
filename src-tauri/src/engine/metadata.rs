//! dr.ini 解析 —— 槽位×章节元数据索引
//!
//! dr.ini 结构（INI 格式）：
//! ```ini
//! [G_5_0]
//! SideB="0.000000"
//! Name="ZZZ"
//! Level="1.000000"
//! ...
//! [URA]
//! 4_2="1.000000"
//! [F_ITEMS]
//! 5_0_0="1.000000"
//! ```

use std::collections::HashMap;
use std::path::Path;

use crate::engine::types::SlotMeta;

/// dr.ini 解析结果
pub struct DrIni {
    /// (chapter, slot) → 元数据
    pub slots: HashMap<(u32, u32), SlotMeta>,
    /// 隐藏 Boss 表：(chapter, slot) → 值
    #[allow(dead_code)]
    pub ura: HashMap<(u32, u32), f64>,
    /// 物品标记表：(chapter, slot, idx) → 值
    #[allow(dead_code)]
    pub f_items: HashMap<(u32, u32, u32), f64>,
}

fn parse_f64(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
}

/// 解析 "4_2" → (4, 2)
fn parse_pair(key: &str) -> Option<(u32, u32)> {
    let mut it = key.split('_');
    let a = it.next()?.parse::<u32>().ok()?;
    let b = it.next()?.parse::<u32>().ok()?;
    Some((a, b))
}

/// 解析 "5_0_0" → (5, 0, 0)
fn parse_triple(key: &str) -> Option<(u32, u32, u32)> {
    let mut it = key.split('_');
    let a = it.next()?.parse::<u32>().ok()?;
    let b = it.next()?.parse::<u32>().ok()?;
    let c = it.next()?.parse::<u32>().ok()?;
    Some((a, b, c))
}

/// 解析段名 "G_5_0" → (5, 0)；旧式 "G0"/"G3" 返回 None
fn parse_g_section(s: &str) -> Option<(u32, u32)> {
    if !s.starts_with("G_") {
        return None;
    }
    parse_pair(&s[2..])
}

/// 解析 dr.ini 文件
pub fn parse_dr_ini(path: &Path) -> Result<DrIni, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("读取 dr.ini 失败: {e}"))?;
    let text = String::from_utf8_lossy(&bytes);

    let mut slots: HashMap<(u32, u32), SlotMeta> = HashMap::new();
    let mut ura = HashMap::new();
    let mut f_items = HashMap::new();

    let mut cur_section = String::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            cur_section = line[1..line.len() - 1].to_string();
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let key = line[..eq].trim();
        let val = line[eq + 1..].trim().trim_matches('"');

        match cur_section.as_str() {
            "URA" => {
                if let Some((ch, slot)) = parse_pair(key) {
                    ura.insert((ch, slot), parse_f64(val));
                }
            }
            "F_ITEMS" => {
                if let Some((ch, slot, idx)) = parse_triple(key) {
                    f_items.insert((ch, slot, idx), parse_f64(val));
                }
            }
            s => {
                if let Some((ch, slot)) = parse_g_section(s) {
                    let meta = slots.entry((ch, slot)).or_insert_with(|| SlotMeta {
                        chapter: ch,
                        slot,
                        ..Default::default()
                    });
                    match key {
                        "Name" => {
                            meta.name = val.to_string();
                            meta.empty = val == "[EMPTY]";
                        }
                        "Level" => meta.level = parse_f64(val),
                        "Love" => meta.love = parse_f64(val),
                        "Time" => meta.time = parse_f64(val),
                        "Date" => meta.date = parse_f64(val),
                        "Room" => meta.room = parse_f64(val),
                        "SideB" => meta.side_b = parse_f64(val),
                        "UraBoss" => meta.ura_boss = parse_f64(val),
                        "Version" => meta.version = val.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(DrIni {
        slots,
        ura,
        f_items,
    })
}
