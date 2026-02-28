import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";
import i18n from "../i18n";
import { getPreferences, getProject } from "../lib/commands";
import { useStore } from "../store";

export function useAppInit() {
	const setPrefs = useStore((s) => s.setPrefs);
	const setProject = useStore((s) => s.setProject);
	const darkMode = useStore((s) => s.prefs.dark_mode);
	const uiScale = useStore((s) => s.prefs.ui_scale);
	const language = useStore((s) => s.prefs.language);

	useEffect(() => {
		Promise.all([getPreferences(), getProject()]).then(([p, proj]) => {
			setPrefs(p);
			setProject(proj);
		});
	}, [setPrefs, setProject]);

	useEffect(() => {
		document.documentElement.classList.toggle("dark", darkMode);
		document.body.classList.toggle("dark", darkMode);
	}, [darkMode]);

	useEffect(() => {
		getCurrentWebviewWindow().setZoom(uiScale ?? 1);
	}, [uiScale]);

	useEffect(() => {
		i18n.changeLanguage(language.toLowerCase());
	}, [language]);
}
