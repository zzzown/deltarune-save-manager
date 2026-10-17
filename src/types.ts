// 与 Rust 端 serde 结构一一对应的类型定义

/** 存档面：A = 正篇，B = 怪异路线（Side B） */
export type Face = "A" | "B";

/** 槽位元数据（来自 dr.ini 的 [G_{ch}_{slot}] 段） */
export interface SlotMeta {
  chapter: number;
  slot: number;
  name: string;
  level: number;
  love: number;
  /** 游戏时间（帧，÷30 = 秒） */
  time: number;
  date: number;
  room: number;
  /** B 面状态：0 / 1 */
  sideB: number;
  /** 隐藏 Boss（裏）状态：0–3 */
  uraBoss: number;
  version: string;
  empty: boolean;
}

/** 单个存档文件条目（来自目录扫描） */
export interface SaveFile {
  /** 文件名，如 filech5_0_b */
  filename: string;
  chapter: number;
  slot: number;
  face: Face;
  path: string;
  size: number;
  lineCount: number;
}

export type Health = "ok" | "warn" | "broken";

/** 一个存档档位（槽位 × 面） */
export interface Profile {
  /** 槽位：0/1/2 */
  slot: number;
  face: Face;
  /** 存在存档文件的章节号（升序） */
  chapters: number[];
  /** 玩家名 */
  name: string;
  level: number;
  time: number;
  sideB: boolean;
  /** 已通关章节（存在 completion file） */
  completedChapters: number[];
  health: Health;
  /** 体检发现的问题描述 */
  issues: string[];
}

/** 扫描结果 */
export interface ScanResult {
  saveDir: string;
  profiles: Profile[];
  files: SaveFile[];
  /** 无法归类的其它文件（如 filech4_2_old） */
  otherFiles: string[];
  gameRunning: boolean;
}

/** 快照元数据 */
export interface SnapshotMeta {
  id: string;
  createdAt: string;
  slot: number;
  face: Face;
  note: string;
  fileCount: number;
  totalSize: number;
}

/** 应用设置 */
export interface Settings {
  /** 游戏 exe 路径（可选） */
  gamePath: string;
  /** 快照库根目录 */
  snapshotDir: string;
}
