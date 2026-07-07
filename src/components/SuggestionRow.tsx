import type { Suggestion } from "../lib/types";
import { Favicon } from "./Favicon";

const SOURCE_LABEL: Record<Suggestion["source"], string> = {
  alias: "Alias",
  history: "History",
  user: "Visited",
  builtin: "Popular",
};

export function SuggestionRow({
  suggestion,
  selected,
  onOpen,
  onHover,
}: {
  suggestion: Suggestion;
  selected: boolean;
  onOpen: () => void;
  onHover: () => void;
}) {
  return (
    <div
      className={`system-row ${selected ? "selected" : ""}`}
      onClick={onOpen}
      onMouseMove={onHover}
      role="option"
      aria-selected={selected}
    >
      <Favicon src={suggestion.favicon} label={suggestion.domain} />
      <div className="item-details">
        <div className="item-name">{suggestion.domain}</div>
        {suggestion.title && <div className="item-path">{suggestion.title}</div>}
      </div>
      <div className="item-tag">{SOURCE_LABEL[suggestion.source] ?? suggestion.source}</div>
      {selected && <kbd className="key-hint">↵</kbd>}
    </div>
  );
}
