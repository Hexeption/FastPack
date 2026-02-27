import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { buildTree, flattenTreeForNav, isFolderKey } from "../lib/tree";
import { useStore } from "../store";
import type { FrameData, Project } from "../types";

interface UseTreeNavArgs {
	scrollRef: React.RefObject<HTMLDivElement | null>;
	project: Project | null;
	allFrames: FrameData[];
	openSources: Set<string>;
	setOpenSources: React.Dispatch<React.SetStateAction<Set<string>>>;
	visualOrder: string[];
	handleSelect: (id: string, ctrlKey: boolean, shiftKey: boolean) => void;
}

export function useTreeNav({
	scrollRef,
	project,
	allFrames,
	openSources,
	setOpenSources,
	visualOrder,
	handleSelect,
}: UseTreeNavArgs) {
	const selectedFrames = useStore((s) => s.selectedFrames);
	const setSelectedFrames = useStore((s) => s.setSelectedFrames);
	const setAnchorFrame = useStore((s) => s.setAnchorFrame);

	const [openFolders, setOpenFolders] = useState<Set<string>>(new Set());
	const [navCursor, setNavCursor] = useState<string | null>(null);

	const navOrder = useMemo(() => {
		if (!project) return [];
		const order: string[] = [];
		for (let i = 0; i < project.sources.length; i++) {
			const srcKey = `__src__${i}`;
			order.push(srcKey);
			if (!openSources.has(srcKey)) continue;
			const src = project.sources[i];
			const srcName = src.path.split(/[\\/]/).pop() ?? src.path;
			const srcFrames =
				project.sources.length === 1
					? allFrames
					: allFrames.filter((f) => f.id.startsWith(`${srcName}/`));
			const tree = buildTree(srcFrames, `${srcName}/`);
			flattenTreeForNav(tree, openFolders, order);
		}
		return order;
	}, [project, allFrames, openSources, openFolders]);

	const visualOrderRef = useRef(visualOrder);
	visualOrderRef.current = visualOrder;

	const navOrderRef = useRef(navOrder);
	navOrderRef.current = navOrder;

	const openSourcesRef = useRef(openSources);
	openSourcesRef.current = openSources;

	const openFoldersRef = useRef(openFolders);
	openFoldersRef.current = openFolders;

	const navCursorRef = useRef<string | null>(null);
	navCursorRef.current = navCursor;

	const toggleFolder = useCallback((path: string) => {
		setOpenFolders((prev) => {
			const next = new Set(prev);
			if (next.has(path)) next.delete(path);
			else next.add(path);
			return next;
		});
	}, []);

	const handleKeyDown = useCallback(
		(e: React.KeyboardEvent<HTMLDivElement>) => {
			const valid = [
				"ArrowUp",
				"ArrowDown",
				"ArrowLeft",
				"ArrowRight",
				"Home",
				"End",
			];
			if (!valid.includes(e.key)) return;
			e.preventDefault();
			const { selectedFrames: cur, anchorFrame: anchor } = useStore.getState();
			const cursor = navCursorRef.current;

			if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
				if (!cursor || !isFolderKey(cursor)) return;
				if (cursor.startsWith("__src__")) {
					const isOpen = openSourcesRef.current.has(cursor);
					if (e.key === "ArrowRight" && !isOpen) {
						setOpenSources((prev) => new Set([...prev, cursor]));
					} else if (e.key === "ArrowLeft" && isOpen) {
						setOpenSources((prev) => {
							const n = new Set(prev);
							n.delete(cursor);
							return n;
						});
					}
				} else {
					const path = cursor.slice("__folder__:".length);
					const isOpen = openFoldersRef.current.has(path);
					if (e.key === "ArrowRight" && !isOpen) {
						setOpenFolders((prev) => new Set([...prev, path]));
					} else if (e.key === "ArrowLeft" && isOpen) {
						setOpenFolders((prev) => {
							const n = new Set(prev);
							n.delete(path);
							return n;
						});
					}
				}
				return;
			}

			if (e.key === "Home" || e.key === "End") {
				if (e.shiftKey) {
					const order = visualOrderRef.current;
					const target = e.key === "Home" ? order[0] : order[order.length - 1];
					if (target) {
						navCursorRef.current = target;
						setNavCursor(target);
						if (anchor) handleSelect(target, false, true);
					}
				} else {
					const order = navOrderRef.current;
					const target = e.key === "Home" ? order[0] : order[order.length - 1];
					if (target) {
						navCursorRef.current = target;
						setNavCursor(target);
						if (!isFolderKey(target)) handleSelect(target, false, false);
						else {
							setSelectedFrames([]);
							setAnchorFrame(null);
						}
					}
				}
				return;
			}

			const dir = e.key === "ArrowDown" ? 1 : -1;

			if (!e.shiftKey) {
				const order = navOrderRef.current;
				if (order.length === 0) return;
				const ref = cursor ?? anchor ?? cur[0] ?? null;
				const idx = ref ? order.indexOf(ref) : dir === 1 ? -1 : order.length;
				const next = order[Math.max(0, Math.min(order.length - 1, idx + dir))];
				if (next) {
					navCursorRef.current = next;
					setNavCursor(next);
					if (!isFolderKey(next)) handleSelect(next, false, false);
					else {
						setSelectedFrames([]);
						setAnchorFrame(null);
					}
				}
			} else {
				const order = visualOrderRef.current;
				if (order.length === 0 || !anchor) return;
				const anchorIdx = order.indexOf(anchor);
				if (anchorIdx === -1) return;
				const curIdxs = cur
					.map((id) => order.indexOf(id))
					.filter((i) => i !== -1);
				if (curIdxs.length === 0) return;
				const movingEnd =
					dir === 1 ? Math.max(...curIdxs) : Math.min(...curIdxs);
				const next =
					order[Math.max(0, Math.min(order.length - 1, movingEnd + dir))];
				if (next) {
					navCursorRef.current = next;
					setNavCursor(next);
					handleSelect(next, false, true);
				}
			}
		},
		[handleSelect, setOpenSources, setSelectedFrames, setAnchorFrame],
	);

	// Scroll selected frame into view
	useEffect(() => {
		if (selectedFrames.length === 0 || !scrollRef.current) return;
		const id = selectedFrames[selectedFrames.length - 1];
		const el = scrollRef.current.querySelector(
			`[data-frame-id="${CSS.escape(id)}"]`,
		);
		if (el) el.scrollIntoView({ block: "nearest" });
	}, [selectedFrames, scrollRef]);

	// Highlight nav cursor with folder color
	useEffect(() => {
		if (!scrollRef.current) return;
		const container = scrollRef.current;
		const prev = container.querySelector("[data-nav-active]");
		if (prev) {
			prev.removeAttribute("data-nav-active");
			(prev as HTMLElement).style.removeProperty("--nav-color");
		}
		if (!navCursor) return;
		const el = container.querySelector(
			`[data-nav-id="${CSS.escape(navCursor)}"]`,
		);
		if (el) {
			el.setAttribute("data-nav-active", "");
			const navColor = el.getAttribute("data-nav-color");
			if (navColor)
				(el as HTMLElement).style.setProperty("--nav-color", navColor);
			el.scrollIntoView({ block: "nearest" });
		}
	}, [navCursor, scrollRef]);

	const clearNav = useCallback(() => {
		navCursorRef.current = null;
		setNavCursor(null);
	}, []);

	return {
		navCursor,
		navOrder,
		openFolders,
		toggleFolder,
		handleKeyDown,
		clearNav,
	};
}
