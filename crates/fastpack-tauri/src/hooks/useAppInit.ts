import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect } from "react";
import i18n from "../i18n";
import { getPreferences, getProject } from "../lib/commands";
import { useStore } from "../store";

export function useAppInit() {
	const setPrefs = useStore((s) => s.setPrefs);
	const setProject = useStore((s) => s.setProject);
	const prefs = useStore((s) => s.prefs);

	useEffect(() => {
		Promise.all([getPreferences(), getProject()]).then(([p, proj]) => {
			setPrefs(p);
			setProject(proj);
		});
	}, [setPrefs, setProject]);

	useEffect(() => {
		document.documentElement.classList.toggle("dark", prefs.dark_mode);
		document.body.classList.toggle("dark", prefs.dark_mode);
	}, [prefs.dark_mode]);

	useEffect(() => {
		getCurrentWebviewWindow().setZoom(prefs.ui_scale ?? 1);
	}, [prefs.ui_scale]);

	useEffect(() => {
		i18n.changeLanguage(prefs.language.toLowerCase());
	}, [prefs.language]);
}
