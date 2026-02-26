import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./store";
import { usePack } from "./hooks/usePack";
import MenuBar from "./components/MenuBar";
import Toolbar from "./components/Toolbar";
import SpriteList from "./components/SpriteList";
import AtlasPreview from "./components/AtlasPreview";
import SettingsPanel from "./components/SettingsPanel";
import OutputLog from "./components/OutputLog";
import PreferencesDialog from "./components/PreferencesDialog";
import type { Preferences, Project } from "./types";
import styles from "./App.module.css";

export default function App() {
  const prefs = useStore((s) => s.prefs);
  const setPrefs = useStore((s) => s.setPrefs);
  const setProject = useStore((s) => s.setProject);
  const prefsOpen = useStore((s) => s.prefsOpen);

  usePack();

  useEffect(() => {
    Promise.all([
      invoke<Preferences>("get_preferences"),
      invoke<Project>("get_project"),
    ]).then(([p, proj]) => {
      setPrefs(p);
      setProject(proj);
    });
  }, [setPrefs, setProject]);

  return (
    <div
      className={styles.root}
      data-theme={prefs.dark_mode ? "dark" : "light"}
    >
      <MenuBar />
      <Toolbar />
      <div className={styles.main}>
        <SpriteList />
        <AtlasPreview />
        <SettingsPanel />
      </div>
      <OutputLog />
      {prefsOpen && <PreferencesDialog />}
    </div>
  );
}
