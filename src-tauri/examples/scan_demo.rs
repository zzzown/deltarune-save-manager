//! 验证示例：扫描所有存档目录，打印槽位矩阵与元数据解析结果
//! 运行：cargo run --example scan_demo

use std::path::PathBuf;

use deltarune_save_manager_lib::engine::scanner;
use deltarune_save_manager_lib::engine::types::Face;

fn main() {
    let dirs = scanner::discover_save_dirs();
    if dirs.is_empty() {
        println!("未找到存档目录");
        return;
    }
    for dir in dirs {
        println!("========== 存档目录: {dir} ==========");
        match scanner::scan(&PathBuf::from(&dir)) {
            Ok(result) => {
                println!("游戏运行中: {}", result.game_running);
                for p in &result.profiles {
                    let face = match p.face {
                        Face::A => "正篇",
                        Face::B => "B面",
                    };
                    println!(
                        "槽位 {} · {} | 章节 {:?} | 玩家 {} | LV {} | 时长 {}帧 | 隐藏Boss {} | 已通关 {:?} | 健康 {:?}",
                        p.slot + 1,
                        face,
                        p.chapters,
                        p.name,
                        p.level,
                        p.time,
                        p.ura_boss,
                        p.completed_chapters,
                        p.health
                    );
                    for i in &p.issues {
                        println!("    ⚠ {i}");
                    }
                }
            }
            Err(e) => println!("扫描失败: {e}"),
        }
        println!();
    }
}
