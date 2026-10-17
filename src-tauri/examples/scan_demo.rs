//! 验证示例：扫描本机真实存档目录，打印槽位矩阵与元数据解析结果
//! 运行：cargo run --example scan_demo

use deltarune_save_manager_lib::engine::scanner;

fn main() {
    match scanner::default_save_dir() {
        Some(dir) => {
            println!("存档目录: {}", dir.display());
            match scanner::scan(&dir) {
                Ok(result) => {
                    println!("游戏运行中: {}", result.game_running);
                    println!();
                    println!("========== 档位（槽位 × 面）==========");
                    for p in &result.profiles {
                        let face = match p.face {
                            deltarune_save_manager_lib::engine::types::Face::A => "正篇",
                            deltarune_save_manager_lib::engine::types::Face::B => "B面",
                        };
                        println!(
                            "槽位 {} · {} | 章节 {:?} | 玩家 {} | LV {} | 时长 {}帧 | B面标志 {} | 已通关 {:?} | 健康 {:?}",
                            p.slot + 1,
                            face,
                            p.chapters,
                            p.name,
                            p.level,
                            p.time,
                            p.side_b,
                            p.completed_chapters,
                            p.health
                        );
                        for i in &p.issues {
                            println!("    ⚠ {}", i);
                        }
                    }
                    println!();
                    println!("========== 文件明细 ==========");
                    for f in &result.files {
                        println!(
                            "{:<14} ch{} slot{} {:?} {} 字节 {} 行",
                            f.filename, f.chapter, f.slot, f.face, f.size, f.line_count
                        );
                    }
                    if !result.other_files.is_empty() {
                        println!();
                        println!("========== 其它文件（只备份不解析）==========");
                        for o in &result.other_files {
                            println!("  {}", o);
                        }
                    }
                }
                Err(e) => println!("扫描失败: {e}"),
            }
        }
        None => println!("未找到存档目录 %LOCALAPPDATA%\\DELTARUNE"),
    }
}
