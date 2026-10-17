//! 验证示例：自动探测游戏路径
//! 运行：cargo run --example detect_demo

use deltarune_save_manager_lib::find_game_path;

fn main() {
    match find_game_path() {
        Some(p) => println!("探测到 DELTARUNE.exe: {p}"),
        None => println!("未探测到 DELTARUNE.exe"),
    }
}
