import { useState } from "react";
import Sidebar from "./components/layout/Sidebar";
import ReaderPane from "./components/layout/ReaderPane";
import { SIDEBAR_WIDTH } from "./lib/constants";

export default function App() {
  const [activeFeedId, setActiveFeedId] = useState<number | null>(null);

  return (
    <div
      className="grid h-screen"
      style={{ gridTemplateColumns: `${SIDEBAR_WIDTH}px 1fr` }}
    >
      <Sidebar activeFeedId={activeFeedId} onSelectFeed={setActiveFeedId} />
      <ReaderPane />
    </div>
  );
}
