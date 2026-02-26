import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import type { Preferences } from "../types";
import styles from "./PreferencesDialog.module.css";

export default function PreferencesDialog() {
  const prefs = useStore((s) => s.prefs);
  const setPrefs = useStore((s) => s.setPrefs);
  const setPrefsOpen = useStore((s) => s.setPrefsOpen);

  const update = (patch: Partial<Preferences>) => {
    const next = { ...prefs, ...patch };
    setPrefs(next);
  };

  const handleSave = async () => {
    await invoke("save_preferences", { prefs });
    setPrefsOpen(false);
  };

  const handleCancel = () => setPrefsOpen(false);

  return (
    <div className={styles.overlay} onClick={(e) => e.target === e.currentTarget && handleCancel()}>
      <div className={styles.dialog}>
        <div className={styles.title}>Preferences</div>
        <div className={styles.body}>
          <div className={styles.row}>
            <label>Dark mode</label>
            <input
              type="checkbox"
              checked={prefs.dark_mode}
              onChange={(e) => update({ dark_mode: e.target.checked })}
            />
          </div>
          <div className={styles.row}>
            <label>Check for updates on startup</label>
            <input
              type="checkbox"
              checked={prefs.auto_check_updates}
              onChange={(e) => update({ auto_check_updates: e.target.checked })}
            />
          </div>
          <div className={styles.row}>
            <label>Language</label>
            <select
              value={prefs.language}
              onChange={(e) => update({ language: e.target.value as Preferences["language"] })}
            >
              <option value="En">English</option>
              <option value="Fr">Français</option>
              <option value="Es">Español</option>
              <option value="De">Deutsch</option>
              <option value="It">Italiano</option>
              <option value="Pt">Português</option>
              <option value="Ja">日本語</option>
              <option value="Zh">中文（简体）</option>
              <option value="Ko">한국어</option>
            </select>
          </div>
        </div>
        <div className={styles.footer}>
          <button onClick={handleCancel}>Cancel</button>
          <button className="primary" onClick={handleSave}>Save</button>
        </div>
      </div>
    </div>
  );
}
