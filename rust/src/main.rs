//! Fantasy Map Generator - 命令行工具入口
//!
//! 这是地图生成器的可执行文件入口点。
//! 它简单地调用 CLI 模块来处理所有的应用程序逻辑。
//!
//! # 使用方法
//!
//! ```bash
//! # 查看帮助
//! map_generation --help
//!
//! # 生成地图
//! map_generation --seed 12345 --cities 5 --towns 10
//! ```
//!
//! # 参考来源
//! - 原始 C++ 实现: src/main.cpp

use fantasy_map_generator::cli;

/// 程序入口点
///
/// 调用 CLI 模块的 run 函数来执行地图生成。
///
/// # 返回
/// - `Ok(())` - 程序成功执行
/// - `Err(...)` - 发生错误
fn main() -> anyhow::Result<()> {
    cli::run()
}
