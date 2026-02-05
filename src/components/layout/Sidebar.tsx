interface SidebarProps {
  activeFeedId: number | null;
  onSelectFeed: (feedId: number | null) => void;
}

export default function Sidebar({ activeFeedId, onSelectFeed }: SidebarProps) {
  return (
    <aside
      className="flex flex-col h-full overflow-y-auto border-r"
      style={{
        backgroundColor: "var(--color-bg-sidebar)",
        borderColor: "var(--color-border)",
      }}
    >
      <div
        className="flex items-center justify-between px-4 py-3 border-b"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h1 className="text-lg font-semibold" style={{ color: "var(--color-text-primary)" }}>
          Boke
        </h1>
        <div className="flex items-center gap-1">
          <button
            type="button"
            aria-label="Refresh all feeds"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Refresh all feeds"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M1 8a7 7 0 0113.24-3.18M15 8A7 7 0 011.76 11.18" />
              <path d="M14.24 1v3.82h-3.82M1.76 15v-3.82h3.82" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Add feed"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Add feed"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M8 3v10M3 8h10" />
            </svg>
          </button>
        </div>
      </div>

      <div className="px-2 py-2">
        <button
          type="button"
          className="w-full flex items-center justify-between px-3 py-2 rounded-md text-sm transition-colors"
          style={{
            backgroundColor: activeFeedId === null ? "var(--color-bg-secondary)" : "transparent",
            color: "var(--color-text-primary)",
          }}
          onClick={() => onSelectFeed(null)}
        >
          <span className="font-medium">All Feeds</span>
        </button>
      </div>

      <div className="flex-1 px-2">
        <div
          className="flex flex-col items-center justify-center h-full text-center px-4"
          style={{ color: "var(--color-text-muted)" }}
        >
          <p className="text-sm">No feeds yet</p>
          <p className="text-xs mt-1">Press + to add your first feed</p>
        </div>
      </div>
    </aside>
  );
}
