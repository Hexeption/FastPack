import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import type { Project } from "../types";
import styles from "./MenuBar.module.css";

export default function MenuBar() {
  const setProject = useStore((s) => s.setProject);
  const dirty = useStore((s) => s.dirty);
  const project = useStore((s) => s.project);
  const setPrefsOpen = useStore((s) => s.setPrefsOpen);
  const prefs = useStore((s) => s.prefs);
  const setPrefs = useStore((s) => s.setPrefs);

  const handleNew = async () => {
    const p = await invoke<Project>("new_project");
    setProject(p);
  };

  const handleOpen = async () => {
    const path = await invoke<string | null>("open_file_dialog");
    if (!path) return;
    try {
      const p = await invoke<Project>("open_project", { path });
      setProject(p);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSave = async () => {
    if (!project) return;
    const path = await invoke<string | null>("save_file_dialog", {
      defaultName: "project.fpsheet",
    });
    if (!path) return;
    await invoke("save_project", { path });
    useStore.getState().setDirty(false);
  };

  const toggleTheme = () => {
    const next = { ...prefs, dark_mode: !prefs.dark_mode };
    setPrefs(next);
    invoke("save_preferences", { prefs: next });
  };

  return (
    <div className={styles.bar}>
      <span className={styles.logo}>FastPack</span>
      {dirty && <span className={styles.dirty}>●</span>}
      <div className={styles.menus}>
        <div className={styles.menu}>
          <span className={styles.menuLabel}>File</span>
          <div className={styles.dropdown}>
            <button onClick={handleNew}>New Project</button>
            <button onClick={handleOpen}>Open Project…</button>
            <button onClick={handleSave} disabled={!project}>Save Project As…</button>
          </div>
        </div>
        <div className={styles.menu}>
          <span className={styles.menuLabel}>View</span>
          <div className={styles.dropdown}>
            <button onClick={toggleTheme}>
              {prefs.dark_mode ? "Light Theme" : "Dark Theme"}
            </button>
            <button onClick={() => setPrefsOpen(true)}>Preferences…</button>
          </div>
        </div>
      </div>
    </div>
  );
}
