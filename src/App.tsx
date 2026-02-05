import { useState, useEffect, useRef, useCallback } from "react";
import Sidebar from "./components/layout/Sidebar";
import ArticleList from "./components/layout/ArticleList";
import ReaderPane from "./components/layout/ReaderPane";
import SearchBar from "./components/search/SearchBar";
import KeyboardShortcuts from "./components/ui/KeyboardShortcuts";
import UpdateBanner from "./components/ui/UpdateBanner";
import { SIDEBAR_WIDTH, ARTICLE_LIST_WIDTH } from "./lib/constants";
import { useFeeds, useRefreshAllFeeds, useRefreshFeed } from "./api/feeds";
import { useArticles, useToggleRead, useToggleFavorite, useMarkAllRead, useMarkAllUnread } from "./api/articles";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { useTheme } from "./hooks/useTheme";
import { openUrl } from "@tauri-apps/plugin-opener";
import { listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import type { Article } from "./types";

export default function App() {
  const [activeFeedId, setActiveFeedId] = useState<number | null>(null);
  const [activeFilter, setActiveFilter] = useState<"all" | "unread" | "favourites">("all");
  const [selectedArticle, setSelectedArticle] = useState<Article | null>(null);
  const [showSearch, setShowSearch] = useState(false);
  const [showAddFeed, setShowAddFeed] = useState(false);
  const [articleListVisible, setArticleListVisible] = useState(true);
  const readerRef = useRef<HTMLElement | null>(null);
  const hasRefreshed = useRef(false);

  const { theme, toggleTheme } = useTheme();
  const { data: feeds } = useFeeds();
  const { data: articles, isLoading: articlesLoading, error: articlesErr } = useArticles(
    activeFeedId,
    activeFilter === "unread",
    activeFilter === "favourites",
  );
  const refreshAll = useRefreshAllFeeds();
  const refreshFeed = useRefreshFeed();
  const toggleRead = useToggleRead();
  const toggleFavorite = useToggleFavorite();
  const markAllRead = useMarkAllRead();
  const markAllUnread = useMarkAllUnread();

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

  // Cmd+K / Ctrl+K / "/" for search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setShowSearch(true);
        return;
      }
      // "/" opens search when not in an input/dialog
      if (
        e.key === "/" &&
        !e.metaKey && !e.ctrlKey && !e.shiftKey &&
        !showSearch && !showAddFeed &&
        !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)
      ) {
        e.preventDefault();
        setShowSearch(true);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showSearch, showAddFeed]);

  const handleSelectArticle = useCallback((article: Article) => {
    setSelectedArticle(article);
    if (!article.is_read) {
      toggleRead.mutate(article.id);
    }
  }, [toggleRead]);

  const handleToggleArticleList = useCallback(() => {
    setArticleListVisible((v) => !v);
  }, []);

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
    onMarkAllRead: () => markAllRead.mutate(activeFeedId),
    onOpenAddFeed: () => setShowAddFeed(true),
    onOpenSearch: () => setShowSearch(true),
    onToggleArticleList: handleToggleArticleList,
  });

  // Reset index when feed or filter changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [activeFeedId, activeFilter, setSelectedIndex]);

  // Always show article list when no article is selected in the reader
  const effectiveArticleListVisible = articleListVisible || !selectedArticle;

  const gridColumns = effectiveArticleListVisible
    ? `${SIDEBAR_WIDTH}px ${ARTICLE_LIST_WIDTH}px 1fr`
    : `${SIDEBAR_WIDTH}px 0px 1fr`;

  return (
    <div
      className="grid h-screen"
      style={{ gridTemplateColumns: gridColumns }}
    >
      <UpdateBanner />
      <Sidebar
        theme={theme}
        onToggleTheme={toggleTheme}
        activeFeedId={activeFeedId}
        onSelectFeed={(feedId) => {
          setActiveFeedId(feedId);
          if (feedId !== null) setActiveFilter("all");
        }}
        activeFilter={activeFilter}
        onSelectFilter={(filter) => {
          setActiveFilter(filter);
          setActiveFeedId(null);
          setSelectedIndex(0);
        }}
        showAddFeed={showAddFeed}
        onOpenAddFeed={() => setShowAddFeed(true)}
        onCloseAddFeed={() => setShowAddFeed(false)}
        onOpenHelp={() => setShowShortcuts(true)}
        articleListVisible={effectiveArticleListVisible}
        onToggleArticleList={handleToggleArticleList}
      />
      <ArticleList
        visible={effectiveArticleListVisible}
        articles={articles ?? []}
        articlesLoading={articlesLoading}
        articlesError={articlesErr ? String(articlesErr) : null}
        refreshError={refreshAll.error ? String(refreshAll.error) : null}
        selectedArticleId={selectedArticle?.id ?? null}
        selectedIndex={selectedArticle ? -1 : selectedIndex}
        onSelectArticle={handleSelectArticle}
        onToggleFavorite={(id) => toggleFavorite.mutate(id)}
        onToggleRead={(id) => toggleRead.mutate(id)}
        onMarkAllRead={() => markAllRead.mutate(activeFeedId)}
        onMarkAllUnread={() => markAllUnread.mutate(activeFeedId)}
      />
      <ReaderPane article={selectedArticle} readerRef={readerRef} onToggleFavorite={(id) => toggleFavorite.mutate(id)} onToggleRead={(id) => toggleRead.mutate(id)} />
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
