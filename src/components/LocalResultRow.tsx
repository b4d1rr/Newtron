import type { AppRouteItem, FileRouteItem } from "../lib/types";

/** Human-readable size, e.g. "4.2 MB". */
function formatSize(bytes: number): string {
  if (bytes <= 0) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let n = bytes;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function LocalResultRow({
  item,
  selected,
  onOpen,
  onHover,
}: {
  item: AppRouteItem | FileRouteItem;
  selected: boolean;
  onOpen: () => void;
  onHover: () => void;
}) {
  const isApp = item.kind === "app";
  const path = isApp ? item.path : item.full_path;
  const tag = isApp ? "App" : item.extension ? item.extension.toUpperCase() : "File";
  const meta = isApp ? path : [item.extension?.toUpperCase(), formatSize(item.size)].filter(Boolean).join(" · ") || path;

  return (
    <div
      className={`system-row ${selected ? "selected" : ""}`}
      onClick={onOpen}
      onMouseMove={onHover}
      role="option"
      aria-selected={selected}
    >
      <div className={`item-icon-box ${isApp ? "icon-app" : "icon-file"}`}>{item.name.charAt(0).toUpperCase()}</div>
      <div className="item-details">
        <div className="item-name">{item.name}</div>
        <div className="item-path">{meta}</div>
      </div>
      <div className="item-tag">{tag}</div>
      {selected && <kbd className="key-hint">↵</kbd>}
    </div>
  );
}
