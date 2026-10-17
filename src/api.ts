import { invoke } from "@tauri-apps/api/core";
import type {
  ScanResult,
  SnapshotMeta,
  Settings,
} from "./types";

/** 扫描存档目录（saveDir 为空则用默认目录） */
export async function scanSaveDir(saveDir?: string): Promise<ScanResult> {
  return invoke<ScanResult>("scan", { saveDir: saveDir ?? null });
}

/** 发现所有 DELTARUNE 存档目录 */
export async function listSaveDirs(): Promise<string[]> {
  return invoke<string[]>("list_save_dirs");
}

/** 列出所有快照 */
export async function listSnapshots(): Promise<SnapshotMeta[]> {
  return invoke<SnapshotMeta[]>("list_snapshots");
}

/** 为一整个档位（槽位×面）创建快照 */
export async function createSnapshot(
  slot: number,
  face: string,
  note: string
): Promise<SnapshotMeta> {
  return invoke<SnapshotMeta>("create_snapshot", { slot, face, note });
}

/** 恢复某个快照 */
export async function restoreSnapshot(id: string): Promise<void> {
  return invoke<void>("restore_snapshot", { id });
}

/** 删除某个快照（软删除，移入回收站） */
export async function deleteSnapshot(id: string): Promise<void> {
  return invoke<void>("delete_snapshot", { id });
}

/** 切换快照星标锁定 */
export async function toggleLock(id: string, locked: boolean): Promise<void> {
  return invoke<void>("toggle_lock", { id, locked });
}

/** 启动游戏 */
export async function launchGame(): Promise<void> {
  return invoke<void>("launch_game");
}

/** 读取设置 */
export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

/** 保存设置 */
export async function setSettings(s: Settings): Promise<void> {
  return invoke<void>("set_settings", { settings: s });
}

/** 自动探测游戏 exe 路径 */
export async function detectGamePath(): Promise<string | null> {
  return invoke<string | null>("detect_game_path");
}
