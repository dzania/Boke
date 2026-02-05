interface KeyboardShortcutsProps {
  onClose: () => void;
}

const SHORTCUTS = [
  { section: "Navigation", bindings: [
    { key: "j / ↓", desc: "Next article" },
    { key: "k / ↑", desc: "Previous article" },
    { key: "Enter / o", desc: "Open article" },
    { key: "Escape", desc: "Close reader / dialog" },
    { key: "g g", desc: "Jump to top of list" },
    { key: "G", desc: "Jump to bottom of list" },
    { key: "1-9", desc: "Switch to nth feed" },
  ]},
  { section: "Reader", bindings: [
    { key: "J", desc: "Scroll down" },
    { key: "K", desc: "Scroll up" },
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
    { key: "/ or Cmd+K", desc: "Search" },
    { key: "?", desc: "Show this help" },
  ]},
];

export default function KeyboardShortcuts({ onClose }: KeyboardShortcutsProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="rounded-lg p-6 w-[480px] max-h-[80vh] overflow-y-auto shadow-xl"
        style={{ backgroundColor: "var(--color-bg-primary)", border: "1px solid var(--color-border)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold" style={{ color: "var(--color-text-primary)" }}>
            Keyboard Shortcuts
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded hover:opacity-70"
            style={{ color: "var(--color-text-muted)" }}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>

        {SHORTCUTS.map(({ section, bindings }) => (
          <div key={section} className="mb-4 last:mb-0">
            <h3
              className="text-xs font-medium uppercase tracking-wider mb-2"
              style={{ color: "var(--color-text-muted)" }}
            >
              {section}
            </h3>
            <div className="space-y-1">
              {bindings.map(({ key, desc }) => (
                <div key={key} className="flex items-center justify-between py-1">
                  <span className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
                    {desc}
                  </span>
                  <kbd
                    className="text-xs px-2 py-0.5 rounded font-mono"
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
