import { useState, useEffect, useRef, useCallback } from "react";
import Sidebar from "./components/layout/Sidebar";
import ReaderPane from "./components/layout/ReaderPane";
import SearchBar from "./components/search/SearchBar";
import KeyboardShortcuts from "./components/ui/KeyboardShortcuts";
import { SIDEBAR_WIDTH } from "./lib/constants";
import { useFeeds, useRefreshAllFeeds, useRefreshFeed } from "./api/feeds";
import { useArticles, useToggleRead, useToggleFavorite, useMarkAllRead } from "./api/articles";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { openUrl } from "@tauri-apps/plugin-opener";
import { listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import type { Article } from "./types";

export default function App() {
  const [activeFeedId, setActiveFeedId] = useState<number | null>(null);
  const [selectedArticle, setSelectedArticle] = useState<Article | null>(null);
  const [showSearch, setShowSearch] = useState(false);
  const [showAddFeed, setShowAddFeed] = useState(false);
  const readerRef = useRef<HTMLElement | null>(null);
  const hasRefreshed = useRef(false);

  const { data: feeds } = useFeeds();
  const { data: articles } = useArticles(activeFeedId);
  const refreshAll = useRefreshAllFeeds();
  const refreshFeed = useRefreshFeed();
  const toggleRead = useToggleRead();
  const toggleFavorite = useToggleFavorite();
  const markAllRead = useMarkAllRead();

  // Refresh all feeds on launch
  useEffect(() => {
    if (!hasRefreshed.current) {
      hasRefreshed.current = true;
      refreshAll.mutate();
    }
  }, []);

  // Listen for new-articles event and show notification
  useEffect(() => {
    const unlisten = listen<{ total: number; feeds: number }>("new-articles", async (event) => {
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      if (granted) {
        sendNotification({
          title: "Boke",
          body: `${event.payload.total} new articles from ${event.payload.feeds} feeds`,
        });
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Listen for tray refresh event
  useEffect(() => {
    const unlisten = listen("tray-refresh", () => {
      refreshAll.mutate();
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [refreshAll]);

  // Cmd+K / Ctrl+K for search (must work even in inputs)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setShowSearch(true);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleSelectArticle = useCallback((article: Article) => {
    setSelectedArticle(article);
    if (!article.is_read) {
      toggleRead.mutate(article.id);
    }
  }, [toggleRead]);

  const { selectedIndex, setSelectedIndex, showShortcuts, setShowShortcuts } = useKeyboardNav({
    articles: articles ?? [],
    selectedArticle,
    feeds: feeds ?? [],
    activeFeedId,
    readerRef,
    disabled: showSearch || showAddFeed,
    onSelectArticle: handleSelectArticle,
    onClearArticle: () => setSelectedArticle(null),
    onSelectFeed: (feedId) => {
      setActiveFeedId(feedId);
      setSelectedIndex(0);
    },
    onToggleRead: (id) => toggleRead.mutate(id),
    onToggleFavorite: (id) => toggleFavorite.mutate(id),
    onOpenInBrowser: (url) => openUrl(url),
    onRefreshFeed: () => {
      if (activeFeedId) refreshFeed.mutate(activeFeedId);
      else refreshAll.mutate();
    },
    onRefreshAll: () => refreshAll.mutate(),
    onMarkAllRead: () => {
      if (activeFeedId) markAllRead.mutate(activeFeedId);
    },
    onOpenAddFeed: () => setShowAddFeed(true),
    onOpenSearch: () => setShowSearch(true),
  });

  // Reset index when feed changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [activeFeedId, setSelectedIndex]);

  return (
    <div
      className="grid h-screen"
      style={{ gridTemplateColumns: `${SIDEBAR_WIDTH}px 1fr` }}
    >
      <Sidebar
        activeFeedId={activeFeedId}
        onSelectFeed={setActiveFeedId}
        selectedArticleId={selectedArticle?.id ?? null}
        selectedIndex={selectedArticle ? -1 : selectedIndex}
        onSelectArticle={handleSelectArticle}
        articles={articles ?? []}
        showAddFeed={showAddFeed}
        onOpenAddFeed={() => setShowAddFeed(true)}
        onCloseAddFeed={() => setShowAddFeed(false)}
      />
      <ReaderPane article={selectedArticle} readerRef={readerRef} />
      {showSearch && (
        <SearchBar
          onSelectArticle={handleSelectArticle}
          onClose={() => setShowSearch(false)}
        />
      )}
      {showShortcuts && (
        <KeyboardShortcuts onClose={() => setShowShortcuts(false)} />
      )}
    </div>
  );
}
