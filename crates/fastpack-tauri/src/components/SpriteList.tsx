import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import type { Project } from "../types";
import styles from "./SpriteList.module.css";

export default function SpriteList() {
  const project = useStore((s) => s.project);
  const setProject = useStore((s) => s.setProject);
  const sheets = useStore((s) => s.sheets);
  const setDirty = useStore((s) => s.setDirty);

  const allFrames = sheets.flatMap((s) => s.frames);

  const handleAddSource = async () => {
    const path = await invoke<string | null>("open_folder_dialog");
    if (!path) return;
    const p = await invoke<Project>("add_source", { path });
    setProject(p);
    setDirty(true);
  };

  const handleRemoveSource = async (index: number) => {
    const p = await invoke<Project>("remove_source", { index });
    setProject(p);
    setDirty(true);
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span>Sprites</span>
        <button className="icon-btn" onClick={handleAddSource} title="Add source folder">+</button>
      </div>
      <div className={styles.body}>
        {!project || project.sources.length === 0 ? (
          <div className={styles.empty}>
            <p>No sources.</p>
            <button onClick={handleAddSource}>Add folder…</button>
          </div>
        ) : (
          project.sources.map((src, i) => {
            const srcName = src.path.split(/[\\/]/).pop() ?? src.path;
            const srcFrames = allFrames.filter((f) =>
              f.id.startsWith(srcName + "/") || allFrames.length <= 100
            );
            return (
              <details key={src.path} className={styles.source} open>
                <summary className={styles.sourceName}>
                  <span>📁 {srcName}</span>
                  <button
                    className="icon-btn"
                    onClick={(e) => { e.preventDefault(); handleRemoveSource(i); }}
                    title="Remove source"
                  >✕</button>
                </summary>
                <div className={styles.frames}>
                  {allFrames.length === 0 ? (
                    <span className={styles.hint}>Pack to preview sprites.</span>
                  ) : (
                    srcFrames.map((f) => (
                      <div key={f.id} className={styles.frame} title={f.id}>
                        <span className={styles.frameId}>{f.id}</span>
                        <span className={styles.frameDim}>{f.w}×{f.h}</span>
                      </div>
                    ))
                  )}
                </div>
              </details>
            );
          })
        )}
      </div>
    </div>
  );
}
