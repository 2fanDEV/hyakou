import type { Hyako, UploadStatusEvent } from "@wasm/hyako_wasm_bindings";
import { AssetInformation } from "@wasm/hyako_wasm_bindings";
import { useCallback, useState } from "react";
import type { FileDirectoryState, UploadStatus } from "#/lib/fileDirectory";
import {
	createFileNode,
	createFolderNode,
	moveNodes,
	updateNodeById,
} from "#/lib/fileDirectory";

const CHECKMARK_DISMISS_MS = 2000;

const emptyState = (): FileDirectoryState => ({ nodes: [] });

export default function useFileDrop() {
	const [state, setState] = useState<FileDirectoryState>(emptyState);

	const onHyakoReady = useCallback((hyako: Hyako) => {
		hyako.setUploadStatusListener((event: UploadStatusEvent) => {
			const status = event.status as UploadStatus;
			setState((prev) => {
				return {
					...prev,
					nodes: updateNodeById(prev.nodes, event.uploadId, (node) =>
						node.kind === "file"
							? { ...node, status, errorMessage: event.message }
							: node,
					),
				};
			});

			if (status === "success") {
				setTimeout(() => {
					setState((prev) => ({
						...prev,
						nodes: updateNodeById(prev.nodes, event.uploadId, (node) =>
							node.kind === "file"
								? { ...node, status: "done" as const }
								: node,
						),
					}));
				}, CHECKMARK_DISMISS_MS);
			}
		});
	}, []);

	const uploadFiles = useCallback(async (hyako: Hyako, fileList: FileList) => {
		const uploads = Array.from(fileList).map(async (file) => {
			const id = crypto.randomUUID();
			setState((prev) => ({
				...prev,
				nodes: [...prev.nodes, createFileNode(id, file.name)],
			}));

			const bytes = new Uint8Array(await file.arrayBuffer());
			hyako.upload_file(
				new AssetInformation(
					id,
					bytes,
					file.name,
					BigInt(file.size),
					file.lastModified,
				),
			);
		});
		await Promise.all(uploads);
	}, []);

	const createDirectory = useCallback((name: string) => {
		setState((prev) => ({
			...prev,
			nodes: [...prev.nodes, createFolderNode(name)],
		}));
	}, []);

	const moveItems = useCallback(
		(itemIds: string[], directoryId: string | null, index: number) => {
			setState((prev) => ({
				...prev,
				nodes: moveNodes(prev.nodes, itemIds, directoryId, index),
			}));
		},
		[],
	);

	return {
		state,
		onHyakoReady,
		uploadFiles,
		createDirectory,
		moveItems,
	};
}
