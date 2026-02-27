import type { FrameData } from "../types";

export type FileNode = { kind: "file"; name: string; frame: FrameData };
export type FolderNode = {
	kind: "folder";
	name: string;
	path: string;
	children: TreeNode[];
};
export type TreeNode = FileNode | FolderNode;

function insertFrame(
	nodes: TreeNode[],
	parts: string[],
	depth: number,
	frame: FrameData,
) {
	if (depth >= parts.length) return;
	if (depth === parts.length - 1) {
		nodes.push({ kind: "file", name: parts[depth], frame });
		return;
	}
	const folderName = parts[depth];
	const folderPath = parts.slice(0, depth + 1).join("/");
	const existing = nodes.find(
		(n): n is FolderNode => n.kind === "folder" && n.name === folderName,
	);
	if (existing) {
		insertFrame(existing.children, parts, depth + 1, frame);
	} else {
		const folder: FolderNode = {
			kind: "folder",
			name: folderName,
			path: folderPath,
			children: [],
		};
		insertFrame(folder.children, parts, depth + 1, frame);
		nodes.push(folder);
	}
}

function sortTree(nodes: TreeNode[]) {
	nodes.sort((a, b) => {
		if (a.kind !== b.kind) return a.kind === "folder" ? -1 : 1;
		return a.name.localeCompare(b.name, undefined, {
			numeric: true,
			sensitivity: "base",
		});
	});
	for (const node of nodes) {
		if (node.kind === "folder") sortTree(node.children);
	}
}

export function buildTree(
	frames: FrameData[],
	stripPrefix: string,
): TreeNode[] {
	const nodes: TreeNode[] = [];
	for (const frame of frames) {
		const id =
			stripPrefix && frame.id.startsWith(stripPrefix)
				? frame.id.slice(stripPrefix.length)
				: frame.id;
		const parts = id.split("/").filter(Boolean);
		if (parts.length > 0) insertFrame(nodes, parts, 0, frame);
	}
	sortTree(nodes);
	return nodes;
}

export function flattenTree(nodes: TreeNode[], out: string[]) {
	for (const node of nodes) {
		if (node.kind === "file") out.push(node.frame.id);
		else flattenTree(node.children, out);
	}
}

export function isFolderKey(s: string) {
	return s.startsWith("__src__") || s.startsWith("__folder__:");
}

export function flattenTreeForNav(
	nodes: TreeNode[],
	openFolders: Set<string>,
	out: string[],
) {
	for (const node of nodes) {
		if (node.kind === "file") {
			out.push(node.frame.id);
		} else {
			out.push(`__folder__:${node.path}`);
			if (openFolders.has(node.path)) {
				flattenTreeForNav(node.children, openFolders, out);
			}
		}
	}
}
