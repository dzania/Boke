interface KeyboardShortcutsProps {
  onClose: () => void;
}

const SHORTCUTS = [
  { section: "Navigation", bindings: [
    { key: "j / ↓", desc: "Next article / scroll down in reader" },
    { key: "k / ↑", desc: "Previous article / scroll up in reader" },
    { key: "Enter / o", desc: "Open selected article" },
    { key: "Escape", desc: "Close reader / dialog" },
    { key: "g g", desc: "Jump to top of list" },
    { key: "G", desc: "Jump to bottom of list" },
    { key: "1-9", desc: "Switch to nth feed" },
  ]},
  { section: "Reader", bindings: [
    { key: "J / K", desc: "Scroll reader (fast)" },
    { key: "Space", desc: "Page down" },
    { key: "Shift+Space", desc: "Page up" },
  ]},
  { section: "Actions", bindings: [
    { key: "m", desc: "Toggle read / unread" },
    { key: "s", desc: "Toggle favorite" },
    { key: "v", desc: "Open in browser" },
    { key: "r", desc: "Refresh current feed" },
    { key: "R", desc: "Refresh all feeds" },
    { key: "Shift+A", desc: "Mark all as read" },
    { key: "a", desc: "Add feed" },
    { key: "b", desc: "Toggle article list panel" },
    { key: "/ or Cmd+K", desc: "Search articles" },
    { key: "?", desc: "Show this help" },
  ]},
];

const TIPS = [
  "Paste any blog URL — Boke auto-discovers the RSS feed.",
  "Right-click a feed to move it to a folder.",
  "Articles are fetched in full when the feed only provides a summary.",
  "Your theme preference is saved between sessions.",
];

export default function KeyboardShortcuts({ onClose }: KeyboardShortcutsProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="rounded-lg p-6 w-[520px] max-h-[80vh] overflow-y-auto shadow-xl"
        style={{ backgroundColor: "var(--color-bg-primary)", border: "1px solid var(--color-border)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-lg font-semibold" style={{ color: "var(--color-text-primary)" }}>
            Help
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded hover:opacity-70"
            style={{ color: "var(--color-text-muted)" }}
            aria-label="Close"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>

        {/* Quick tips */}
        <div className="mb-5 pb-5 border-b" style={{ borderColor: "var(--color-border)" }}>
          <h3
            className="text-xs font-medium uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Tips
          </h3>
          <ul className="space-y-1.5">
            {TIPS.map((tip) => (
              <li
                key={tip}
                className="text-sm flex items-start gap-2"
                style={{ color: "var(--color-text-secondary)" }}
              >
                <span className="shrink-0 mt-0.5" style={{ color: "var(--color-accent)" }}>
                  &bull;
                </span>
                {tip}
              </li>
            ))}
          </ul>
        </div>

        {/* Keyboard shortcuts */}
        <h3
          className="text-xs font-medium uppercase tracking-wider mb-3"
          style={{ color: "var(--color-text-muted)" }}
        >
          Keyboard Shortcuts
        </h3>
        {SHORTCUTS.map(({ section, bindings }) => (
          <div key={section} className="mb-4 last:mb-0">
            <h4
              className="text-[11px] font-medium uppercase tracking-wider mb-1.5"
              style={{ color: "var(--color-text-muted)", opacity: 0.7 }}
            >
              {section}
            </h4>
            <div className="space-y-1">
              {bindings.map(({ key, desc }) => (
                <div key={key} className="flex items-center justify-between py-0.5">
                  <span className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
                    {desc}
                  </span>
                  <kbd
                    className="text-xs px-2 py-0.5 rounded font-mono ml-4 shrink-0"
                    style={{
                      backgroundColor: "var(--color-bg-secondary)",
                      color: "var(--color-text-primary)",
                      border: "1px solid var(--color-border)",
                    }}
                  >
                    {key}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
