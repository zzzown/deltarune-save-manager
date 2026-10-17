import { useState } from "react";
import type { Settings } from "../types";

export function SettingsPanel({
  initial,
  onSave,
  onClose,
  onDetect,
}: {
  initial: Settings;
  onSave: (s: Settings) => void;
  onClose: () => void;
  onDetect: () => Promise<string | null>;
}) {
  const [gamePath, setGamePath] = useState(initial.gamePath);
  const [snapshotDir, setSnapshotDir] = useState(initial.snapshotDir);
  const [detecting, setDetecting] = useState(false);

  const detect = async () => {
    setDetecting(true);
    try {
      const p = await onDetect();
      if (p) setGamePath(p);
    } finally {
      setDetecting(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2 className="modal-title">
          <span className="brand-star">*</span> 设置
        </h2>

        <div className="field">
          <label>游戏路径（DELTARUNE.exe）</label>
          <div className="field-row">
            <input
              className="input"
              value={gamePath}
              onChange={(e) => setGamePath(e.target.value)}
              placeholder="留空则启动时自动探测 Steam 安装"
            />
            <button className="btn" onClick={detect} disabled={detecting}>
              {detecting ? "探测中…" : "自动探测"}
            </button>
          </div>
          <span className="hint">例如 D:\Steam\steamapps\common\DELTARUNE\DELTARUNE.exe</span>
        </div>

        <div className="field">
          <label>快照目录</label>
          <input
            className="input"
            value={snapshotDir}
            onChange={(e) => setSnapshotDir(e.target.value)}
            placeholder="留空则使用默认目录（%APPDATA%\DeltaruneSaveManager\snapshots）"
          />
        </div>

        <div className="modal-actions">
          <button className="btn" onClick={onClose}>
            取消
          </button>
          <button
            className="btn btn-primary"
            onClick={() =>
              onSave({
                gamePath: gamePath.trim(),
                snapshotDir: snapshotDir.trim(),
              })
            }
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
