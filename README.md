# DELTARUNE 存档管理器

一个**不注入、不改游戏本体**的 Deltarune 外置存档时间线备份工具。以「槽位 × 章节 × 面」为管理单元，对整档做原子快照、时间线回退与一致性体检。

> 状态：开发中（MVP 骨架已就绪，待 Rust 工具链编译验证）。
> 技术方案见 [docs/方案.md](docs/方案.md)，功能对齐参考见 [docs/Chronicle-参考与功能对齐.md](docs/Chronicle-参考与功能对齐.md)。

## 为什么需要它

Deltarune 的一个「存档槽」横跨多个章节、分散在多个文件里，还区分正篇与第五章怪异路线（B 面 / Side B）。手动备份极易只拷一半导致进度错乱。本工具把这些文件抽象为一个档位，一次备份 / 恢复整列。

## 特性（规划）

- 槽位 × 章节 × 面（A/B）三维矩阵扫描
- `dr.ini` 元数据解析（玩家名 / LV / 时长 / SideB / UraBoss）
- 整档原子快照 + 事务式恢复（恢复前自动备份）
- B 面（Side B / 怪异路线）存档完整兼容 `filech5_{0..9}_b`
- 进程守卫（游戏运行中禁止写入）
- 星标锁定、软删除、多存档目录发现（规划中）

## 技术栈

- Rust + Tauri 2
- React + TypeScript + Vite
- 仅 Windows

## 开发

需要 Node.js 22+、Rust 工具链、WebView2。

```bash
npm install
npm run tauri dev
```

## 存档格式要点

存档位于 `%LOCALAPPDATA%\DELTARUNE`，纯文本、CRLF、一行一值、无校验和，字段靠行号定位。第 1 章 10,318 行，第 2–5 章 3,055 行。Ch5 比 Ch4 多「粉币(1865)」「花币(1964)」字段。详见 [docs/方案.md](docs/方案.md)。
