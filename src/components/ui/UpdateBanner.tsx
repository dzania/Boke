import { useState, useEffect } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";

export default function UpdateBanner() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    check()
      .then(setUpdate)
      .catch(() => {});
  }, []);

  if (!update) return null;

  const handleInstall = async () => {
    setInstalling(true);
    try {
      await update.downloadAndInstall();
    } catch {
      setInstalling(false);
    }
  };

  return (
    <div className="fixed top-0 left-0 right-0 z-50 flex items-center justify-center gap-3 bg-sky-600 px-4 py-2 text-sm text-white">
      <span>Boke {update.version} is available</span>
      <button
        onClick={handleInstall}
        disabled={installing}
        className="rounded bg-white/20 px-3 py-0.5 hover:bg-white/30 disabled:opacity-50"
      >
        {installing ? "Installing…" : "Update now"}
      </button>
    </div>
  );
}
