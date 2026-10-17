import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "./api";
import type { Profile, ScanResult, Settings, SnapshotMeta } from "./types";
import { SlotCard } from "./components/SlotCard";
import { SnapshotTimeline } from "./components/SnapshotTimeline";
import { SettingsPanel } from "./components/SettingsPanel";
import { ConfirmDialog } from "./components/ConfirmDialog";

export default function App() {
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [snapshots, setSnapshots] = useState<SnapshotMeta[]>([]);
  const [selected, setSelected] = useState<{ slot: number; face: string } | null>(
    null
  );
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saveDirs, setSaveDirs] = useState<string[]>([]);
  const [currentDir, setCurrentDir] = useState("");
  const currentDirRef = useRef("");
  const [trash, setTrash] = useState<SnapshotMeta[]>([]);
  const [view, setView] = useState<"snapshots" | "trash">("snapshots");
  const [confirm, setConfirm] = useState<{
    message: string;
    confirmText?: string;
    danger?: boolean;
    action: () => void;
  } | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [s, snaps] = await Promise.all([
        api.scanSaveDir(currentDirRef.current || undefined),
        api.listSnapshots(),
      ]);
      setScan(s);
      setSnapshots(snaps);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const s = await api.getSettings();
        setSettings(s);
        document.documentElement.setAttribute(
          "data-theme",
          s.theme || "green-white"
        );
        const dirs = await api.listSaveDirs();
        const merged = [...new Set([...dirs, ...(s.customDirs || [])])];
        setSaveDirs(merged);
      } catch (e) {
        setError(String(e));
      }
      refresh();
    })();
  }, [refresh]);

  const selectedProfile: Profile | undefined = scan?.profiles.find(
    (p) => p.slot === selected?.slot && p.face === selected?.face
  );

  const selectedSnapshots = snapshots.filter(
    (s) => s.slot === selected?.slot && s.face === selected?.face
  );

  const doCreateSnapshot = async () => {
    if (!selected) return;
    setBusy(true);
    setError("");
    try {
      await api.createSnapshot(selected.slot, selected.face, note.trim());
      setNote("");
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const doRestore = async (id: string) => {
    setBusy(true);
    setError("");
    try {
      await api.restoreSnapshot(id);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const doLaunch = async () => {
    setError("");
    try {
      await api.launchGame();
    } catch (e) {
      setError(String(e));
    }
  };

  const openSettings = async () => {
    try {
      const s = await api.getSettings();
      setSettings(s);
      setShowSettings(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const saveSettings = async (s: Settings) => {
    setBusy(true);
    setError("");
    try {
      await api.setSettings(s);
      setSettings(s);
      document.documentElement.setAttribute(
        "data-theme",
        s.theme || "green-white"
      );
      const dirs = await api.listSaveDirs();
      const merged = [...new Set([...dirs, ...(s.customDirs || [])])];
      setSaveDirs(merged);
      setShowSettings(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const switchDir = async (dir: string) => {
    currentDirRef.current = dir;
    setCurrentDir(dir);
    await refresh();
  };

  const doToggleLock = async (id: string, locked: boolean) => {
    setBusy(true);
    setError("");
    try {
      await api.toggleLock(id, locked);
      const snaps = await api.listSnapshots();
      setSnapshots(snaps);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadTrash = async () => {
    try {
      const t = await api.listTrash();
      setTrash(t);
    } catch (e) {
      setError(String(e));
    }
  };

  const doRestoreTrash = async (id: string) => {
    setBusy(true);
    try {
      await api.restoreTrash(id);
      await loadTrash();
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const doPurgeTrash = async (id: string) => {
    setBusy(true);
    try {
      await api.purgeTrash(id);
      await loadTrash();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleView = async () => {
    if (view === "snapshots") {
      await loadTrash();
      setView("trash");
    } else {
      setView("snapshots");
    }
  };

  const askConfirm = (
    message: string,
    action: () => void,
    opts?: { confirmText?: string; danger?: boolean }
  ) => {
    setConfirm({ message, action, ...opts });
  };

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-star">*</span>
          <span>DELTARUNE 存档管理器</span>
        </div>
        <div className="topbar-right">
          <select
            className="select"
            value={currentDir}
            onChange={(e) => switchDir(e.target.value)}
            disabled={busy}
            title="切换存档目录"
          >
            <option value="">默认目录</option>
            {saveDirs.map((d) => (
              <option key={d} value={d}>
                {d.split(/[\\/]/).pop()}
              </option>
            ))}
          </select>
          {scan?.gameRunning && (
            <span className="tag tag-danger">游戏运行中 · 已锁定写入</span>
          )}
          <button className="btn" onClick={() => refresh()} disabled={busy}>
            刷新
          </button>
          <button className="btn" onClick={doLaunch} disabled={busy}>
            启动游戏
          </button>
          <button className="btn" onClick={openSettings} disabled={busy}>
            设置
          </button>
        </div>
      </header>

      {error && <div className="banner banner-danger">{error}</div>}

      <main className="layout">
        <section className="matrix">
          <div className="section-title">
            <h2>存档档位</h2>
            <span className="hint">按「槽位 × 面」管理整列存档</span>
          </div>

          {scan && scan.profiles.length === 0 && (
            <div className="empty">
              未在 {scan.saveDir} 中发现存档。
            </div>
          )}

          <div className="matrix-grid">
            {scan?.profiles.map((p) => (
              <SlotCard
                key={`${p.slot}-${p.face}`}
                profile={p}
                selected={
                  selected?.slot === p.slot && selected?.face === p.face
                }
                onClick={() => setSelected({ slot: p.slot, face: p.face })}
              />
            ))}
          </div>

          {scan && scan.otherFiles.length > 0 && (
            <div className="others">
              <span className="hint">其它文件（只备份不解析）：</span>
              {scan.otherFiles.join("、")}
            </div>
          )}
        </section>

        <aside className="side">
          <div className="section-title">
            <h2>{view === "snapshots" ? "快照" : "回收站"}</h2>
            {view === "snapshots" && selectedProfile && (
              <span className="hint">
                槽位 {selectedProfile.slot + 1} ·{" "}
                {selectedProfile.face === "B" ? "B 面" : "正篇"}
              </span>
            )}
            <button
              className="btn btn-small btn-ghost"
              style={{ marginLeft: "auto" }}
              onClick={toggleView}
            >
              {view === "snapshots" ? "回收站" : "返回快照"}
            </button>
          </div>

          {view === "snapshots" ? (
            selected ? (
              <>
                <div className="snapshot-bar">
                  <input
                    className="input"
                    placeholder="备注（可选）"
                    value={note}
                    onChange={(e) => setNote(e.target.value)}
                  />
                  <button
                    className="btn btn-primary"
                    onClick={doCreateSnapshot}
                    disabled={busy}
                  >
                    创建快照
                  </button>
                </div>
                <SnapshotTimeline
                  snapshots={selectedSnapshots}
                  onRestore={(id) =>
                    askConfirm(
                      "确定恢复此快照吗？恢复前会自动备份当前存档。",
                      () => doRestore(id)
                    )
                  }
                  onDelete={(id) =>
                    askConfirm("移入回收站？（可在回收站中找回）", async () => {
                      await api.deleteSnapshot(id);
                      await refresh();
                    })
                  }
                  onToggleLock={doToggleLock}
                  busy={busy}
                />
              </>
            ) : (
              <div className="empty">选择一个档位查看其快照。</div>
            )
          ) : trash.length === 0 ? (
            <div className="empty">回收站是空的。</div>
          ) : (
            <ul className="timeline">
              {trash.map((t) => (
                <li className="snapshot" key={t.id}>
                  <div className="snapshot-main">
                    <div className="snapshot-note">
                      {t.note || "（无备注）"}
                    </div>
                    <div className="snapshot-meta">
                      槽位 {t.slot + 1} · {t.face === "B" ? "B 面" : "正篇"} ·{" "}
                      {new Date(t.createdAt).toLocaleString()}
                    </div>
                  </div>
                  <div className="snapshot-actions">
                    <button
                      className="btn btn-small"
                      disabled={busy}
                      onClick={() => doRestoreTrash(t.id)}
                    >
                      恢复
                    </button>
                    <button
                      className="btn btn-small btn-ghost"
                      disabled={busy}
                      onClick={() =>
                        askConfirm(
                          "彻底删除？此操作不可撤销。",
                          () => doPurgeTrash(t.id),
                          { danger: true, confirmText: "彻底删除" }
                        )
                      }
                    >
                      彻底删除
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </aside>
      </main>

      {showSettings && settings && (
        <SettingsPanel
          initial={settings}
          onSave={saveSettings}
          onClose={() => setShowSettings(false)}
          onDetect={api.detectGamePath}
        />
      )}

      {confirm && (
        <ConfirmDialog
          message={confirm.message}
          confirmText={confirm.confirmText}
          danger={confirm.danger}
          onConfirm={() => {
            confirm.action();
            setConfirm(null);
          }}
          onCancel={() => setConfirm(null)}
        />
      )}
    </div>
  );
}
