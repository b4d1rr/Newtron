export function WebSearchRow({
  query,
  selected,
  onOpen,
  onHover,
}: {
  query: string;
  selected: boolean;
  onOpen: () => void;
  onHover: () => void;
}) {
  return (
    <div
      className={`web-search-row ${selected ? "selected" : ""}`}
      onClick={onOpen}
      onMouseMove={onHover}
      role="option"
      aria-selected={selected}
    >
      <div className="item-icon-box icon-web">⌕</div>
      <div className="item-details">
        <div className="item-name">
          Search the web for <span className="web-search-query">&ldquo;{query}&rdquo;</span>
        </div>
        <div className="item-path">Opens in your default browser</div>
      </div>
      {selected && <kbd className="key-hint">↵</kbd>}
    </div>
  );
}
