import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import styles from "./Toolbar.module.css";

export default function Toolbar() {
  const isPacking = useStore((s) => s.isPacking);
  const isWatching = useStore((s) => s.isWatching);
  const project = useStore((s) => s.project);
  const setIsWatching = useStore((s) => s.setIsWatching);

  const hasSources = (project?.sources.length ?? 0) > 0;

  const handlePack = () => {
    invoke("pack").catch(console.error);
  };

  const handleWatch = async () => {
    if (isWatching) {
      await invoke("stop_watch");
      setIsWatching(false);
    } else {
      await invoke("start_watch");
      setIsWatching(true);
    }
  };

  return (
    <div className={styles.bar}>
      <button
        className="primary"
        onClick={handlePack}
        disabled={isPacking || !hasSources}
      >
        {isPacking ? "Packing…" : "⚡ Pack"}
      </button>
      <button
        onClick={handleWatch}
        disabled={!hasSources}
        style={isWatching ? { borderColor: "var(--accent)", color: "var(--accent)" } : {}}
      >
        {isWatching ? "👁 Watching" : "👁 Watch"}
      </button>
    </div>
  );
}
