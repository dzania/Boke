import { useState } from "react";
import { openFileDialog } from "../../lib/platform";
import { useAddFeed, useImportOpml } from "../../api/feeds";

interface AddFeedDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function AddFeedDialog({ open: isOpen, onClose }: AddFeedDialogProps) {
  const [url, setUrl] = useState("");
  const [importStatus, setImportStatus] = useState<string | null>(null);
  const addFeed = useAddFeed();
  const importOpml = useImportOpml();

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;
    addFeed.mutate(url.trim(), {
      onSuccess: () => {
        setUrl("");
        onClose();
      },
    });
  };

  const handleImportOpml = async () => {
    const fileOrPath = await openFileDialog({
      filters: [{ name: "OPML", extensions: ["opml", "xml"] }],
    });
    if (!fileOrPath) return;

    setImportStatus(null);
    importOpml.mutate(fileOrPath, {
      onSuccess: (result) => {
        const parts: string[] = [];
        if (result.added > 0) parts.push(`${result.added} feeds added`);
        if (result.skipped > 0) parts.push(`${result.skipped} already subscribed`);
        if (result.errors.length > 0) parts.push(`${result.errors.length} failed`);
        setImportStatus(parts.join(", "));
      },
    });
  };

  const busy = addFeed.isPending || importOpml.isPending;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="rounded-lg p-6 w-[420px] shadow-xl"
        style={{
          backgroundColor: "var(--color-bg-primary)",
          border: "1px solid var(--color-border)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-semibold mb-4" style={{ color: "var(--color-text-primary)" }}>
          Add Feed
        </h2>
        <form onSubmit={handleSubmit}>
          <input
            type="url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://example.com or feed URL"
            autoFocus
            className="w-full px-3 py-2 rounded-md text-sm outline-none"
            style={{
              backgroundColor: "var(--color-bg-secondary)",
              color: "var(--color-text-primary)",
              border: "1px solid var(--color-border)",
            }}
          />
          {addFeed.isError && (
            <p className="text-red-500 text-xs mt-2">
              {addFeed.error instanceof Error ? addFeed.error.message : String(addFeed.error)}
            </p>
          )}
          <div className="flex justify-end gap-2 mt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-3 py-1.5 rounded-md text-sm"
              style={{ color: "var(--color-text-secondary)" }}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={busy || !url.trim()}
              className="px-3 py-1.5 rounded-md text-sm font-medium text-white disabled:opacity-50"
              style={{ backgroundColor: "var(--color-accent)" }}
            >
              {addFeed.isPending ? "Adding..." : "Add"}
            </button>
          </div>
        </form>

        {/* Divider */}
        <div className="flex items-center gap-3 my-4">
          <div className="flex-1 h-px" style={{ backgroundColor: "var(--color-border)" }} />
          <span className="text-xs" style={{ color: "var(--color-text-muted)" }}>
            or
          </span>
          <div className="flex-1 h-px" style={{ backgroundColor: "var(--color-border)" }} />
        </div>

        {/* Import OPML */}
        <button
          type="button"
          disabled={busy}
          onClick={handleImportOpml}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-md text-sm transition-opacity hover:opacity-80 disabled:opacity-50"
          style={{
            backgroundColor: "var(--color-bg-secondary)",
            color: "var(--color-text-primary)",
            border: "1px solid var(--color-border)",
          }}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <path d="M8 10V2M8 2L5 5M8 2l3 3" />
            <path d="M2 10v3a1 1 0 001 1h10a1 1 0 001-1v-3" />
          </svg>
          {importOpml.isPending ? "Importing..." : "Import OPML file"}
        </button>

        {importOpml.isError && (
          <p className="text-red-500 text-xs mt-2">
            {importOpml.error instanceof Error
              ? importOpml.error.message
              : String(importOpml.error)}
          </p>
        )}
        {importStatus && (
          <p className="text-xs mt-2" style={{ color: "var(--color-text-secondary)" }}>
            {importStatus}
          </p>
        )}
      </div>
    </div>
  );
}
