import { useState, useEffect, useLayoutEffect, useRef, useCallback, useMemo } from "react";
import Sidebar from "./components/layout/Sidebar";
import ArticleList from "./components/layout/ArticleList";
import ReaderPane from "./components/layout/ReaderPane";
import SearchBar from "./components/search/SearchBar";
import KeyboardShortcuts from "./components/ui/KeyboardShortcuts";
import UpdateBanner from "./components/ui/UpdateBanner";
import { SIDEBAR_WIDTH, ARTICLE_LIST_WIDTH } from "./lib/constants";
import { useFeeds, useRefreshAllFeeds, useRefreshFeed } from "./api/feeds";
import { useFolders } from "./api/folders";
import {
  useArticles,
  useSetRead,
  useSetFavorite,
  useMarkAllRead,
  useMarkAllUnread,
} from "./api/articles";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { useTheme } from "./hooks/useTheme";
import { buildSidebarItems } from "./lib/sidebarItems";
import {
  openUrl,
  listen,
  isNotificationPermissionGranted,
  requestNotificationPermission,
  sendNotification,
} from "./lib/platform";
import type { ArticleFilter, ArticleListItem, RefreshResult } from "./types";

const NO_ARTICLES: ArticleListItem[] = [];

function logError(context: string) {
  return (error: unknown) => console.error(context, error);
}

function describeRefreshFailure(error: unknown, results: RefreshResult[] | undefined) {
  if (error) return String(error);
  const failedCount = results?.filter((result) => result.error !== null).length ?? 0;
  if (failedCount === 0) return null;
  return failedCount === 1
    ? "1 feed could not be refreshed"
    : `${failedCount} feeds could not be refreshed`;
}

export default function App() {
  const [activeFeedId, setActiveFeedId] = useState<number | null>(null);
  const [activeFilter, setActiveFilter] = useState<ArticleFilter>("all");
  // Snapshot of the opened article. The live copy is looked up in the list cache below, so
  // flags stay current; the snapshot covers articles opened from search.
  const [selection, setSelection] = useState<ArticleListItem | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showSearch, setShowSearch] = useState(false);
  const [showAddFeed, setShowAddFeed] = useState(false);
  const [articleListVisible, setArticleListVisible] = useState(true);
  const [collapsedFolders, setCollapsedFolders] = useState<Set<number>>(() => new Set());
  const readerRef = useRef<HTMLElement | null>(null);

  const { theme, toggleTheme } = useTheme();
  const { data: feeds } = useFeeds();
  const { data: folders } = useFolders();
  const articlesQuery = useArticles(activeFeedId, activeFilter);
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = articlesQuery;
  const articles = useMemo(
    () => articlesQuery.data?.pages.flat() ?? NO_ARTICLES,
    [articlesQuery.data]
  );
  const selectedArticle = useMemo(
    () => (selection ? (articles.find((a) => a.id === selection.id) ?? selection) : null),
    [selection, articles]
  );

  // `mutate` functions are stable across renders; the mutation result objects are not, so
  // callbacks depend only on `mutate`.
  const refreshAll = useRefreshAllFeeds();
  const { mutate: refreshAllMutate, isPending: refreshAllPending } = refreshAll;
  const { mutate: refreshFeedMutate } = useRefreshFeed();
  const { mutate: setRead } = useSetRead();
  const { mutate: setFavorite } = useSetFavorite();
  const { mutate: markAllReadMutate } = useMarkAllRead();
  const { mutate: markAllUnreadMutate } = useMarkAllUnread();

  const startRefreshAll = useCallback(() => {
    if (!refreshAllPending) refreshAllMutate();
  }, [refreshAllPending, refreshAllMutate]);

  // Event listeners are registered once and call the latest refresh through this ref.
  const startRefreshAllRef = useRef(startRefreshAll);
  useLayoutEffect(() => {
    startRefreshAllRef.current = startRefreshAll;
  });

  // Refresh all feeds on launch. The ref guards against StrictMode's double effect run.
  const hasRefreshed = useRef(false);
  useEffect(() => {
    if (hasRefreshed.current) return;
    hasRefreshed.current = true;
    startRefreshAllRef.current();
  }, []);

  useEffect(() => {
    const unlisten = listen<{ total: number; feeds: number }>("new-articles", async (event) => {
      let granted = await isNotificationPermissionGranted();
      if (!granted) granted = await requestNotificationPermission();
      if (granted) {
        await sendNotification(
          "Boke",
          `${event.payload.total} new articles from ${event.payload.feeds} feeds`
        );
      }
    });
    return () => {
      unlisten.then((stop) => stop()).catch(logError("Failed to remove listener"));
    };
  }, []);

  useEffect(() => {
    const unlisten = listen("tray-refresh", () => startRefreshAllRef.current());
    return () => {
      unlisten.then((stop) => stop()).catch(logError("Failed to remove listener"));
    };
  }, []);

  const patchSelection = useCallback((id: number, patch: Partial<ArticleListItem>) => {
    setSelection((current) => (current?.id === id ? { ...current, ...patch } : current));
  }, []);

  const handleSelectArticle = useCallback(
    (article: ArticleListItem) => {
      if (article.is_read) {
        setSelection(article);
        return;
      }
      setSelection({ ...article, is_read: true });
      setRead({ article, isRead: true });
    },
    [setRead]
  );

  const handleClearArticle = useCallback(() => setSelection(null), []);

  const handleToggleRead = useCallback(
    (article: ArticleListItem) => {
      const isRead = !article.is_read;
      setRead({ article, isRead });
      patchSelection(article.id, { is_read: isRead });
    },
    [setRead, patchSelection]
  );

  const handleToggleFavorite = useCallback(
    (article: ArticleListItem) => {
      const isFavorite = !article.is_favorite;
      setFavorite({ article, isFavorite });
      patchSelection(article.id, { is_favorite: isFavorite });
    },
    [setFavorite, patchSelection]
  );

  const handleSelectFeed = useCallback((feedId: number | null) => {
    setActiveFeedId(feedId);
    if (feedId !== null) setActiveFilter("all");
    setSelectedIndex(0);
  }, []);

  const handleSelectFilter = useCallback((filter: ArticleFilter) => {
    setActiveFilter(filter);
    setActiveFeedId(null);
    setSelectedIndex(0);
  }, []);

  const handleMarkAllRead = useCallback(() => {
    markAllReadMutate({ feedId: activeFeedId, favoritesOnly: activeFilter === "favourites" });
  }, [markAllReadMutate, activeFeedId, activeFilter]);

  const handleMarkAllUnread = useCallback(() => {
    markAllUnreadMutate({ feedId: activeFeedId, favoritesOnly: activeFilter === "favourites" });
  }, [markAllUnreadMutate, activeFeedId, activeFilter]);

  const handleRefreshCurrent = useCallback(() => {
    if (activeFeedId !== null) refreshFeedMutate(activeFeedId);
    else startRefreshAll();
  }, [activeFeedId, refreshFeedMutate, startRefreshAll]);

  const handleLoadMore = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) {
      fetchNextPage().catch(logError("Failed to load more articles"));
    }
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const handleOpenInBrowser = useCallback((url: string) => {
    openUrl(url).catch(logError("Failed to open URL"));
  }, []);

  const handleToggleArticleList = useCallback(() => setArticleListVisible((v) => !v), []);
  const openAddFeed = useCallback(() => setShowAddFeed(true), []);
  const closeAddFeed = useCallback(() => setShowAddFeed(false), []);
  const openSearch = useCallback(() => setShowSearch(true), []);
  const closeSearch = useCallback(() => setShowSearch(false), []);

  const toggleFolderCollapsed = useCallback((folderId: number) => {
    setCollapsedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(folderId)) next.delete(folderId);
      else next.add(folderId);
      return next;
    });
  }, []);

  const sidebarItems = useMemo(
    () => buildSidebarItems(feeds ?? [], folders ?? [], collapsedFolders),
    [feeds, folders, collapsedFolders]
  );

  const { focusZone, sidebarIndex, showShortcuts, setShowShortcuts } = useKeyboardNav({
    articles,
    selectedIndex,
    setSelectedIndex,
    selectedArticle,
    activeFeedId,
    activeFilter,
    readerRef,
    disabled: showSearch || showAddFeed,
    sidebarItems,
    onSelectArticle: handleSelectArticle,
    onClearArticle: handleClearArticle,
    onSelectFeed: handleSelectFeed,
    onSelectFilter: handleSelectFilter,
    onToggleFolder: toggleFolderCollapsed,
    onToggleRead: handleToggleRead,
    onToggleFavorite: handleToggleFavorite,
    onOpenInBrowser: handleOpenInBrowser,
    onRefreshFeed: handleRefreshCurrent,
    onRefreshAll: startRefreshAll,
    onMarkAllRead: handleMarkAllRead,
    onOpenAddFeed: openAddFeed,
    onOpenSearch: openSearch,
    onToggleArticleList: handleToggleArticleList,
    onLoadMore: handleLoadMore,
  });

  const openHelp = useCallback(() => setShowShortcuts(true), [setShowShortcuts]);
  const closeHelp = useCallback(() => setShowShortcuts(false), [setShowShortcuts]);

  const gridColumns = articleListVisible
    ? `${SIDEBAR_WIDTH}px ${ARTICLE_LIST_WIDTH}px 1fr`
    : `${SIDEBAR_WIDTH}px 0px 1fr`;

  return (
    <div className="grid h-screen overflow-hidden" style={{ gridTemplateColumns: gridColumns }}>
      <UpdateBanner />
      <Sidebar
        theme={theme}
        onToggleTheme={toggleTheme}
        activeFeedId={activeFeedId}
        onSelectFeed={handleSelectFeed}
        activeFilter={activeFilter}
        onSelectFilter={handleSelectFilter}
        onRefreshAll={startRefreshAll}
        showAddFeed={showAddFeed}
        onOpenAddFeed={openAddFeed}
        onCloseAddFeed={closeAddFeed}
        onOpenHelp={openHelp}
        articleListVisible={articleListVisible}
        onToggleArticleList={handleToggleArticleList}
        collapsedFolders={collapsedFolders}
        onToggleFolder={toggleFolderCollapsed}
        focusZone={focusZone}
        sidebarIndex={sidebarIndex}
        sidebarItems={sidebarItems}
      />
      <ArticleList
        visible={articleListVisible}
        articles={articles}
        articlesLoading={articlesQuery.isPending}
        articlesError={articlesQuery.error ? String(articlesQuery.error) : null}
        refreshError={describeRefreshFailure(refreshAll.error, refreshAll.data)}
        hasFeeds={(feeds?.length ?? 0) > 0}
        hasMore={hasNextPage}
        loadingMore={isFetchingNextPage}
        selectedArticleId={selectedArticle?.id ?? null}
        selectedIndex={selectedArticle ? -1 : selectedIndex}
        onSelectArticle={handleSelectArticle}
        onToggleFavorite={handleToggleFavorite}
        onToggleRead={handleToggleRead}
        onMarkAllRead={handleMarkAllRead}
        onMarkAllUnread={handleMarkAllUnread}
        onLoadMore={handleLoadMore}
      />
      <ReaderPane
        article={selectedArticle}
        readerRef={readerRef}
        onToggleFavorite={handleToggleFavorite}
        onToggleRead={handleToggleRead}
        theme={theme}
      />
      {showSearch && <SearchBar onSelectArticle={handleSelectArticle} onClose={closeSearch} />}
      {showShortcuts && <KeyboardShortcuts onClose={closeHelp} />}
    </div>
  );
}
