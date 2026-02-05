import { useState, useEffect, useRef } from "react";
import Sidebar from "./components/layout/Sidebar";
import ReaderPane from "./components/layout/ReaderPane";
import { SIDEBAR_WIDTH } from "./lib/constants";
import { useRefreshAllFeeds } from "./api/feeds";
import { useToggleRead } from "./api/articles";
import type { Article } from "./types";

export default function App() {
  const [activeFeedId, setActiveFeedId] = useState<number | null>(null);
  const [selectedArticle, setSelectedArticle] = useState<Article | null>(null);
  const refreshAll = useRefreshAllFeeds();
  const toggleRead = useToggleRead();
  const hasRefreshed = useRef(false);

  // Refresh all feeds on launch
  useEffect(() => {
    if (!hasRefreshed.current) {
      hasRefreshed.current = true;
      refreshAll.mutate();
    }
  }, []);

  const handleSelectArticle = (article: Article) => {
    setSelectedArticle(article);
    if (!article.is_read) {
      toggleRead.mutate(article.id);
    }
  };

  return (
    <div
      className="grid h-screen"
      style={{ gridTemplateColumns: `${SIDEBAR_WIDTH}px 1fr` }}
    >
      <Sidebar
        activeFeedId={activeFeedId}
        onSelectFeed={setActiveFeedId}
        selectedArticleId={selectedArticle?.id ?? null}
        onSelectArticle={handleSelectArticle}
      />
      <ReaderPane article={selectedArticle} />
    </div>
  );
}
