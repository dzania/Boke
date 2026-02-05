import { useState, useMemo } from "react";
import { useFeeds, useRefreshAllFeeds } from "../../api/feeds";
import { useFavoritesCount } from "../../api/articles";
import { useFolders, useCreateFolder, useDeleteFolder, useMoveFeedToFolder } from "../../api/folders";
import AddFeedDialog from "../feed/AddFeedDialog";
import { FeedListSkeleton } from "../ui/Skeleton";
import type { FeedWithMeta } from "../../types";

type FilterType = "all" | "unread" | "favourites";

interface SidebarProps {
  theme: "light" | "dark";
  onToggleTheme: () => void;
  activeFeedId: number | null;
  onSelectFeed: (feedId: number | null) => void;
  activeFilter: FilterType;
  onSelectFilter: (filter: FilterType) => void;
  showAddFeed: boolean;
  onOpenAddFeed: () => void;
  onCloseAddFeed: () => void;
  onOpenHelp: () => void;
  articleListVisible: boolean;
  onToggleArticleList: () => void;
}

function SmartGroupButton({ icon, label, active, badge, onClick }: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  badge?: number;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors mt-0.5"
      style={{
        backgroundColor: active ? "var(--color-bg-secondary)" : "transparent",
        color: "var(--color-text-primary)",
      }}
      onClick={onClick}
    >
      <span className="flex items-center gap-2">
        <span style={{ color: active ? "var(--color-accent)" : "var(--color-text-muted)" }}>{icon}</span>
        <span className={active ? "font-medium" : ""}>{label}</span>
      </span>
      {badge !== undefined && (
        <span className="text-xs" style={{ color: "var(--color-text-muted)" }}>
          {badge}
        </span>
      )}
    </button>
  );
}

export default function Sidebar({
  theme, onToggleTheme,
  activeFeedId, onSelectFeed,
  activeFilter, onSelectFilter,
  showAddFeed, onOpenAddFeed, onCloseAddFeed, onOpenHelp,
  articleListVisible, onToggleArticleList,
}: SidebarProps) {
  const [feedsCollapsed, setFeedsCollapsed] = useState(false);
  const [collapsedFolders, setCollapsedFolders] = useState<Set<number>>(new Set());
  const [newFolderName, setNewFolderName] = useState("");
  const [showNewFolder, setShowNewFolder] = useState(false);
  const [draggingFeedId, setDraggingFeedId] = useState<number | null>(null);
  const [dropTargetFolderId, setDropTargetFolderId] = useState<number | null | "root">(null);
  const { data: feeds } = useFeeds();
  const { data: folders } = useFolders();
  const { data: favoritesCount } = useFavoritesCount();
  const refreshAll = useRefreshAllFeeds();
  const createFolder = useCreateFolder();
  const deleteFolder = useDeleteFolder();
  const moveFeed = useMoveFeedToFolder();

  const totalUnread = feeds?.reduce((sum, f) => sum + f.unread_count, 0) ?? 0;

  const toggleFolderCollapsed = (folderId: number) => {
    setCollapsedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(folderId)) next.delete(folderId);
      else next.add(folderId);
      return next;
    });
  };

  // Group feeds by folder
  const { folderGroups, unfolderedFeeds } = useMemo(() => {
    if (!feeds) return { folderGroups: [], unfolderedFeeds: [] };
    const unfoldered = feeds.filter((f) => !f.folder_id);
    const groups = (folders ?? []).map((folder) => ({
      folder,
      feeds: feeds.filter((f) => f.folder_id === folder.id),
    }));
    return { folderGroups: groups, unfolderedFeeds: unfoldered };
  }, [feeds, folders]);

  const handleCreateFolder = (e: React.FormEvent) => {
    e.preventDefault();
    const name = newFolderName.trim();
    if (!name) return;
    createFolder.mutate(name, {
      onSuccess: () => {
        setNewFolderName("");
        setShowNewFolder(false);
      },
    });
  };

  const renderFeedButton = (feed: FeedWithMeta) => (
    <button
      key={feed.id}
      type="button"
      draggable
      className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors mt-0.5 group"
      style={{
        backgroundColor: activeFeedId === feed.id ? "var(--color-bg-secondary)" : "transparent",
        color: "var(--color-text-primary)",
        opacity: draggingFeedId === feed.id ? 0.5 : 1,
        cursor: draggingFeedId ? "grabbing" : "grab",
      }}
      onClick={() => onSelectFeed(feed.id)}
      onDragStart={(e) => {
        setDraggingFeedId(feed.id);
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", String(feed.id));
      }}
      onDragEnd={() => {
        setDraggingFeedId(null);
        setDropTargetFolderId(null);
      }}
    >
      <span className="flex items-center gap-2 truncate mr-2">
        {feed.favicon_url ? (
          <img
            src={feed.favicon_url}
            alt=""
            width={14}
            height={14}
            className="shrink-0 rounded-sm"
            onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
          />
        ) : (
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" className="shrink-0 opacity-30">
            <circle cx="8" cy="8" r="6" />
            <path d="M8 5v3l2 1" />
          </svg>
        )}
        <span className="truncate">{feed.title}</span>
      </span>
      {feed.unread_count > 0 && (
        <span className="text-xs shrink-0" style={{ color: "var(--color-text-muted)" }}>
          {feed.unread_count}
        </span>
      )}
    </button>
  );

  return (
    <aside
      className="flex flex-col h-full overflow-hidden border-r"
      style={{
        backgroundColor: "var(--color-bg-sidebar)",
        borderColor: "var(--color-border)",
      }}
      aria-label="Sidebar"
    >
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b shrink-0"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h1 className="text-lg font-semibold" style={{ color: "var(--color-text-primary)" }}>
          Boke
        </h1>
        <div className="flex items-center gap-1">
          <button
            type="button"
            aria-label={articleListVisible ? "Hide article list" : "Show article list"}
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: articleListVisible ? "var(--color-accent)" : "var(--color-text-secondary)" }}
            title={articleListVisible ? "Hide article list (b)" : "Show article list (b)"}
            onClick={onToggleArticleList}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <rect x="1.5" y="2" width="13" height="12" rx="1.5" />
              <line x1="6" y1="2" x2="6" y2="14" />
            </svg>
          </button>
          <button
            type="button"
            aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title={theme === "dark" ? "Light mode" : "Dark mode"}
            onClick={onToggleTheme}
          >
            {theme === "dark" ? (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="8" cy="8" r="3.5" />
                <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.4 3.4l.7.7M11.9 11.9l.7.7M3.4 12.6l.7-.7M11.9 4.1l.7-.7" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M13.5 9.2A5.5 5.5 0 016.8 2.5 6.5 6.5 0 1013.5 9.2z" />
              </svg>
            )}
          </button>
          <button
            type="button"
            aria-label="Refresh all feeds"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Refresh all feeds"
            onClick={() => refreshAll.mutate()}
            disabled={refreshAll.isPending}
          >
            <svg
              width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"
              className={refreshAll.isPending ? "animate-spin" : ""}
            >
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
            onClick={onOpenAddFeed}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M8 3v10M3 8h10" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Help & keyboard shortcuts"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Help (?)"
            onClick={onOpenHelp}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="8" cy="8" r="6.5" />
              <path d="M6 6.5a2 2 0 013.94.5c0 1-1.44 1.5-1.94 2M8 11.5v.01" />
            </svg>
          </button>
        </div>
      </div>

      {/* Smart Groups */}
      <div className="px-2 py-2 shrink-0 border-b" style={{ borderColor: "var(--color-border)" }}>
        <SmartGroupButton
          icon={<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M2 4h12M2 8h12M2 12h12" /></svg>}
          label="All Feeds"
          active={activeFeedId === null && activeFilter === "all"}
          badge={totalUnread > 0 ? totalUnread : undefined}
          onClick={() => onSelectFilter("all")}
        />
        <SmartGroupButton
          icon={<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M8 1.5l2.1 4.2 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.4.8-4.7L1.2 6.4l4.7-.7z" /></svg>}
          label="Favourites"
          active={activeFeedId === null && activeFilter === "favourites"}
          badge={favoritesCount && favoritesCount > 0 ? favoritesCount : undefined}
          onClick={() => onSelectFilter("favourites")}
        />
        <SmartGroupButton
          icon={<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="8" cy="8" r="6.5" /><circle cx="8" cy="8" r="2.5" fill="currentColor" stroke="none" /></svg>}
          label="Unread"
          active={activeFeedId === null && activeFilter === "unread"}
          badge={totalUnread > 0 ? totalUnread : undefined}
          onClick={() => onSelectFilter("unread")}
        />
      </div>

      {/* Feed List */}
      <nav className="flex-1 overflow-y-auto" aria-label="Feeds">
        <div className="flex items-center justify-between px-4 py-1.5">
          <button
            type="button"
            className="flex items-center gap-1 text-xs font-medium uppercase tracking-wider"
            style={{ color: "var(--color-text-muted)" }}
            onClick={() => setFeedsCollapsed((v) => !v)}
          >
            <svg
              width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2"
              className="transition-transform"
              style={{ transform: feedsCollapsed ? "rotate(-90deg)" : "rotate(0deg)" }}
            >
              <path d="M4 6l4 4 4-4" />
            </svg>
            <span>Feeds</span>
          </button>
          <button
            type="button"
            aria-label="New folder"
            title="New folder"
            className="p-0.5 rounded hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-muted)" }}
            onClick={() => setShowNewFolder((v) => !v)}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M2 4.5A1.5 1.5 0 013.5 3h3l1.5 2h4.5A1.5 1.5 0 0114 6.5v5a1.5 1.5 0 01-1.5 1.5h-9A1.5 1.5 0 012 11.5z" />
              <path d="M8 7.5v3M6.5 9h3" />
            </svg>
          </button>
        </div>

        {/* New folder input */}
        {showNewFolder && (
          <form onSubmit={handleCreateFolder} className="px-3 pb-2">
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder="Folder name"
              autoFocus
              className="w-full px-2 py-1 rounded text-xs outline-none"
              style={{
                backgroundColor: "var(--color-bg-secondary)",
                color: "var(--color-text-primary)",
                border: "1px solid var(--color-border)",
              }}
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  setShowNewFolder(false);
                  setNewFolderName("");
                }
              }}
            />
          </form>
        )}

        {!feedsCollapsed && (
          <div className="px-2 pb-2">
            {/* Folders */}
            {folderGroups.map(({ folder, feeds: folderFeeds }) => (
              <div key={folder.id} className="mt-1">
                <div
                  className="flex items-center group/folder rounded-md transition-colors"
                  style={{
                    borderLeft: dropTargetFolderId === folder.id ? "2px solid var(--color-accent)" : "2px solid transparent",
                    backgroundColor: dropTargetFolderId === folder.id ? "var(--color-bg-secondary)" : "transparent",
                  }}
                  onDragOver={(e) => {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = "move";
                    if (dropTargetFolderId !== folder.id) setDropTargetFolderId(folder.id);
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    const feedId = Number(e.dataTransfer.getData("text/plain"));
                    if (feedId) {
                      moveFeed.mutate({ feedId, folderId: folder.id });
                    }
                    setDraggingFeedId(null);
                    setDropTargetFolderId(null);
                  }}
                >
                  <button
                    type="button"
                    className="flex-1 flex items-center gap-1.5 px-2 py-1 text-xs font-medium uppercase tracking-wider"
                    style={{ color: "var(--color-text-muted)" }}
                    onClick={() => toggleFolderCollapsed(folder.id)}
                  >
                    <svg
                      width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2"
                      className="transition-transform shrink-0"
                      style={{ transform: collapsedFolders.has(folder.id) ? "rotate(-90deg)" : "rotate(0deg)" }}
                    >
                      <path d="M4 6l4 4 4-4" />
                    </svg>
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" className="shrink-0 opacity-50">
                      <path d="M2 4.5A1.5 1.5 0 013.5 3h3l1.5 2h4.5A1.5 1.5 0 0114 6.5v5a1.5 1.5 0 01-1.5 1.5h-9A1.5 1.5 0 012 11.5z" />
                    </svg>
                    <span className="truncate">{folder.name}</span>
                  </button>
                  <button
                    type="button"
                    aria-label="Delete folder"
                    className="p-0.5 rounded opacity-0 group-hover/folder:opacity-60 hover:!opacity-100 transition-opacity mr-1"
                    style={{ color: "var(--color-text-muted)" }}
                    title="Delete folder"
                    onClick={() => deleteFolder.mutate(folder.id)}
                  >
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                      <path d="M4 4l8 8M12 4l-8 8" />
                    </svg>
                  </button>
                </div>
                {!collapsedFolders.has(folder.id) && (
                  <div className="ml-3">
                    {folderFeeds.map(renderFeedButton)}
                    {folderFeeds.length === 0 && (
                      <p className="text-xs px-3 py-1 italic" style={{ color: "var(--color-text-muted)", opacity: 0.5 }}>
                        Empty
                      </p>
                    )}
                  </div>
                )}
              </div>
            ))}

            {/* Unfoldered feeds — also a drop zone for removing from folder */}
            <div
              className="rounded-md transition-colors"
              style={{
                borderLeft: dropTargetFolderId === "root" ? "2px solid var(--color-accent)" : "2px solid transparent",
                backgroundColor: dropTargetFolderId === "root" ? "var(--color-bg-secondary)" : "transparent",
                minHeight: draggingFeedId ? "32px" : undefined,
              }}
              onDragOver={(e) => {
                e.preventDefault();
                e.dataTransfer.dropEffect = "move";
                if (dropTargetFolderId !== "root") setDropTargetFolderId("root");
              }}
              onDrop={(e) => {
                e.preventDefault();
                const feedId = Number(e.dataTransfer.getData("text/plain"));
                if (feedId) {
                  moveFeed.mutate({ feedId, folderId: null });
                }
                setDraggingFeedId(null);
                setDropTargetFolderId(null);
              }}
            >
              {unfolderedFeeds.map(renderFeedButton)}
              {draggingFeedId && unfolderedFeeds.length === 0 && (
                <p className="text-xs px-3 py-1.5 italic" style={{ color: "var(--color-text-muted)", opacity: 0.6 }}>
                  Drop here to remove from folder
                </p>
              )}
            </div>

            {!feeds && <FeedListSkeleton />}
            {feeds && feeds.length === 0 && (
              <div className="text-center py-6" style={{ color: "var(--color-text-muted)" }}>
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="mx-auto mb-2 opacity-40">
                  <path d="M4 11a9 9 0 0 1 9 9M4 4a16 16 0 0 1 16 16M5 20a1 1 0 1 0 0-2 1 1 0 0 0 0 2z" />
                </svg>
                <p className="text-sm">No feeds yet</p>
                <p className="text-xs mt-1">
                  Press <kbd className="px-1 py-0.5 rounded text-[10px]" style={{ backgroundColor: "var(--color-bg-secondary)" }}>a</kbd> or <kbd className="px-1 py-0.5 rounded text-[10px]" style={{ backgroundColor: "var(--color-bg-secondary)" }}>+</kbd> to add your first feed
                </p>
              </div>
            )}
          </div>
        )}
      </nav>

      <AddFeedDialog open={showAddFeed} onClose={onCloseAddFeed} />
    </aside>
  );
}
