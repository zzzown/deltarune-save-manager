import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
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
  const [theme, setTheme] = useState(initial.theme || "green-white");
  const [launchMethod, setLaunchMethod] = useState(
    initial.launchMethod || "direct"
  );
  const [customDirs, setCustomDirs] = useState<string[]>(
    initial.customDirs || []
  );
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

  const pickFile = async () => {
    const p = await open({ multiple: false, directory: false });
    if (typeof p === "string") setGamePath(p);
  };

  const pickDir = async () => {
    const p = await open({ directory: true, multiple: false });
    if (typeof p === "string") setSnapshotDir(p);
  };

  const addCustomDir = async () => {
    const p = await open({ directory: true, multiple: false });
    if (typeof p === "string" && !customDirs.includes(p)) {
      setCustomDirs([...customDirs, p]);
    }
  };

  const removeCustomDir = (d: string) => {
    setCustomDirs(customDirs.filter((x) => x !== d));
  };

  return (
    <div className="modal-overlay">
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
              placeholder="留空则启动时自动探测"
            />
            <button className="btn" onClick={detect} disabled={detecting}>
              {detecting ? "探测中…" : "自动探测"}
            </button>
            <button className="btn" onClick={pickFile}>
              浏览…
            </button>
          </div>
        </div>

        <div className="field">
          <label>启动方式</label>
          <select
            className="select"
            value={launchMethod}
            onChange={(e) => setLaunchMethod(e.target.value)}
          >
            <option value="direct">直接启动游戏</option>
            <option value="steam">通过 Steam 启动</option>
          </select>
        </div>

        <div className="field">
          <label>快照目录</label>
          <div className="field-row">
            <input
              className="input"
              value={snapshotDir}
              onChange={(e) => setSnapshotDir(e.target.value)}
              placeholder="留空则使用默认目录"
              title="默认：%APPDATA%\DeltaruneSaveManager\snapshots"
            />
            <button className="btn" onClick={pickDir}>
              浏览…
            </button>
          </div>
        </div>

        <div className="field">
          <label>自定义存档目录</label>
          {customDirs.map((d) => (
            <div className="field-row dir-row" key={d}>
              <span className="dir-item">{d}</span>
              <button
                className="btn btn-small btn-ghost"
                onClick={() => removeCustomDir(d)}
              >
                移除
              </button>
            </div>
          ))}
          <button className="btn btn-small" onClick={addCustomDir}>
            + 添加目录
          </button>
        </div>

        <div className="field">
          <label>主题</label>
          <select
            className="select"
            value={theme}
            onChange={(e) => setTheme(e.target.value)}
          >
            <option value="green-white">绿白（浅色）</option>
            <option value="dark-yellow">黄黑（深色）</option>
          </select>
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
                theme,
                launchMethod,
                customDirs,
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
