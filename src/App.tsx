import { useCallback, useEffect, useState } from "react";
import * as api from "./api";
import type { Profile, ScanResult, SnapshotMeta } from "./types";
import { SlotCard } from "./components/SlotCard";
import { SnapshotTimeline } from "./components/SnapshotTimeline";

export default function App() {
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [snapshots, setSnapshots] = useState<SnapshotMeta[]>([]);
  const [selected, setSelected] = useState<{ slot: number; face: string } | null>(
    null
  );
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [s, snaps] = await Promise.all([
        api.scanSaveDir(),
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
    refresh();
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

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-star">*</span>
          <span>DELTARUNE 存档管理器</span>
        </div>
        <div className="topbar-right">
          {scan?.gameRunning && (
            <span className="tag tag-danger">游戏运行中 · 已锁定写入</span>
          )}
          <button className="btn" onClick={refresh} disabled={busy}>
            刷新
          </button>
          <button className="btn" onClick={doLaunch} disabled={busy}>
            启动游戏
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
            <h2>快照</h2>
            {selectedProfile && (
              <span className="hint">
                槽位 {selectedProfile.slot + 1} ·{" "}
                {selectedProfile.face === "B" ? "B 面" : "正篇"}
              </span>
            )}
          </div>

          {selected ? (
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
                onRestore={doRestore}
                onDelete={async (id) => {
                  await api.deleteSnapshot(id);
                  await refresh();
                }}
                busy={busy}
              />
            </>
          ) : (
            <div className="empty">选择一个档位查看其快照。</div>
          )}
        </aside>
      </main>
    </div>
  );
}
