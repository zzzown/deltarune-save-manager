import type { Profile } from "../types";

function fmtTime(frames: number): string {
  const sec = Math.floor(frames / 30);
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${m}:${ss}`;
}

export function SlotCard({
  profile,
  selected,
  onClick,
}: {
  profile: Profile;
  selected: boolean;
  onClick: () => void;
}) {
  const healthLabel = {
    ok: "正常",
    warn: "警告",
    broken: "疑似破损",
  }[profile.health];

  return (
    <button
      className={`card ${selected ? "card-selected" : ""}`}
      onClick={onClick}
    >
      <div className="card-head">
        <span className="card-title">
          槽位 {profile.slot + 1}
          {profile.face === "B" && <span className="tag tag-b">B 面</span>}
        </span>
        <span className={`dot dot-${profile.health}`} title={healthLabel} />
      </div>

      <div className="card-name">
        {profile.name || "（空档）"} · LV {profile.level}
      </div>

      <div className="chapter-row">
        {[1, 2, 3, 4, 5, 6, 7].map((ch) => {
          const has = profile.chapters.includes(ch);
          const done = profile.completedChapters.includes(ch);
          return (
            <span
              key={ch}
              className={`chip ${done ? "chip-done" : has ? "chip-has" : "chip-empty"}`}
            >
              {ch}
            </span>
          );
        })}
      </div>

      <div className="card-foot">
        <span>{fmtTime(profile.time)}</span>
        {profile.issues.length > 0 && (
          <span className="warn-text" title={profile.issues.join("\n")}>
            {profile.issues.length} 项提醒
          </span>
        )}
      </div>
    </button>
  );
}
