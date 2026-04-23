export type UploadStatus = "pending" | "success" | "done" | "error";

export type FileNode = {
	kind: "file";
	id: string;
	name: string;
	status: UploadStatus;
	errorMessage?: string;
};

export type FolderNode = {
	kind: "folder";
	id: string;
	name: string;
	children: FileDirectoryNode[];
};

export type FileDirectoryState = {
	nodes: FileDirectoryNode[];
};

export type FileDirectoryNode = FileNode | FolderNode;

export function createFileNode(id: string, name: string): FileNode {
	return {
		kind: "file",
		id,
		name,
		status: "pending",
	};
}

export function createFolderNode(name: string): FolderNode {
	return {
		kind: "folder",
		id: crypto.randomUUID(),
		name,
		children: [],
	};
}

export function updateNodeById(
	nodes: FileDirectoryNode[],
	id: string,
	updater: (node: FileDirectoryNode) => FileDirectoryNode,
): FileDirectoryNode[] {
	let changed = false;
	const nextNodes = nodes.map((node) => {
		if (node.id === id) {
			changed = true;
			return updater(node);
		}

		if (node.kind === "folder") {
			const nextChildren = updateNodeById(node.children, id, updater);
			if (nextChildren !== node.children) {
				changed = true;
				return { ...node, children: nextChildren };
			}
		}

		return node;
	});

	return changed ? nextNodes : nodes;
}

function removeNodesById(
	nodes: FileDirectoryNode[],
	dragIds: Set<string>,
): {
	nodes: FileDirectoryNode[];
	removed: FileDirectoryNode[];
} {
	const removed: FileDirectoryNode[] = [];
	let changed = false;

	const nextNodes = nodes.flatMap((node) => {
		if (dragIds.has(node.id)) {
			removed.push(node);
			changed = true;
			return [];
		}

		if (node.kind === "folder") {
			const result = removeNodesById(node.children, dragIds);
			if (result.nodes !== node.children) {
				changed = true;
				removed.push(...result.removed);
				return [{ ...node, children: result.nodes }];
			}
		}

		return [node];
	});

	return {
		nodes: changed ? nextNodes : nodes,
		removed,
	};
}

function insertNodesAt(
	nodes: FileDirectoryNode[],
	parentId: string | null,
	index: number,
	items: FileDirectoryNode[],
): FileDirectoryNode[] {
	if (parentId === null) {
		const nextNodes = [...nodes];
		nextNodes.splice(index, 0, ...items);
		return nextNodes;
	}

	let changed = false;
	const nextNodes = nodes.map((node) => {
		if (node.kind !== "folder") return node;
		if (node.id === parentId) {
			changed = true;
			const children = [...node.children];
			children.splice(index, 0, ...items);
			return { ...node, children };
		}

		const children = insertNodesAt(node.children, parentId, index, items);
		if (children !== node.children) {
			changed = true;
			return { ...node, children };
		}

		return node;
	});

	return changed ? nextNodes : nodes;
}

export function moveNodes(
	nodes: FileDirectoryNode[],
	dragIds: string[],
	parentId: string | null,
	index: number,
): FileDirectoryNode[] {
	const { nodes: prunedNodes, removed } = removeNodesById(
		nodes,
		new Set(dragIds),
	);
	if (removed.length === 0) return nodes;
	return insertNodesAt(prunedNodes, parentId, index, removed);
}
