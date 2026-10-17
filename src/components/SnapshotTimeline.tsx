import type { SnapshotMeta } from "../types";

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function fmtDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(
    d.getHours()
  )}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function SnapshotTimeline({
  snapshots,
  onRestore,
  onDelete,
  onToggleLock,
  busy,
}: {
  snapshots: SnapshotMeta[];
  onRestore: (id: string) => void;
  onDelete: (id: string) => void;
  onToggleLock: (id: string, locked: boolean) => void;
  busy: boolean;
}) {
  if (snapshots.length === 0) {
    return <div className="empty">该档位还没有快照。</div>;
  }

  return (
    <ul className="timeline">
      {snapshots.map((s) => (
        <li key={s.id} className="snapshot">
          <div className="snapshot-main">
            <div className="snapshot-note">
              {s.locked && <span className="lock-star">★</span>}
              {s.note || "（无备注）"}
            </div>
            <div className="snapshot-meta">
              {fmtDate(s.createdAt)} · {s.fileCount} 个文件 ·{" "}
              {fmtSize(s.totalSize)}
            </div>
          </div>
          <div className="snapshot-actions">
            <button
              className="btn btn-small btn-lock"
              title={s.locked ? "取消星标" : "星标锁定（不占保留配额、不自动清理）"}
              disabled={busy}
              onClick={() => onToggleLock(s.id, !s.locked)}
            >
              {s.locked ? "★" : "☆"}
            </button>
            <button
              className="btn btn-small"
              disabled={busy}
              onClick={() => {
                if (
                  window.confirm(
                    "确定恢复此快照吗？恢复前会自动为当前存档再建一份快照。"
                  )
                ) {
                  onRestore(s.id);
                }
              }}
            >
              恢复
            </button>
            <button
              className="btn btn-small btn-ghost"
              disabled={busy}
              onClick={() => {
                if (window.confirm("移入回收站？（可在回收站中找回）")) {
                  onDelete(s.id);
                }
              }}
            >
              回收
            </button>
          </div>
        </li>
      ))}
    </ul>
  );
}
