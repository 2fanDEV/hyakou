import type { Hyako, UploadStatusEvent } from "@wasm/hyako_wasm_bindings";
import { AssetInformation } from "@wasm/hyako_wasm_bindings";
import { useCallback, useState } from "react";
import type {
	FileDirectory,
	FileDirectoryItem,
	FileDirectoryState,
	UploadStatus,
} from "#/lib/fileDirectory";

const CHECKMARK_DISMISS_MS = 2000;

const emptyState = (): FileDirectoryState => ({ items: [], directories: [] });

export default function useFileDrop() {
	const [state, setState] = useState<FileDirectoryState>(emptyState);

	const onHyakoReady = useCallback((hyako: Hyako) => {
		hyako.setUploadStatusListener((event: UploadStatusEvent) => {
			const status = event.status as UploadStatus;
			setState((prev) => {
				const items = prev.items.map((item) =>
					item.id === event.uploadId
						? {
								...item,
								status,
								errorMessage: event.message,
							}
						: item,
				);
				return { ...prev, items };
			});

			if (status === "success") {
				setTimeout(() => {
					setState((prev) => ({
						...prev,
						items: prev.items.map((item) =>
							item.id === event.uploadId
								? { ...item, status: "done" as const }
								: item,
						),
					}));
				}, CHECKMARK_DISMISS_MS);
			}
		});
	}, []);

	const uploadFiles = useCallback(async (hyako: Hyako, fileList: FileList) => {
		const uploads = Array.from(fileList).map(async (file) => {
			const id = crypto.randomUUID();
			const pending: FileDirectoryItem = {
				id,
				fileName: file.name,
				status: "pending",
			};
			setState((prev) => ({ ...prev, items: [...prev.items, pending] }));

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
		const directory: FileDirectory = { id: crypto.randomUUID(), name };
		setState((prev) => ({
			...prev,
			directories: [...prev.directories, directory],
		}));
	}, []);

	const moveItemToDirectory = useCallback(
		(itemId: string, directoryId: string | undefined) => {
			setState((prev) => ({
				...prev,
				items: prev.items.map((item) =>
					item.id === itemId ? { ...item, directoryId } : item,
				),
			}));
		},
		[],
	);

	return {
		state,
		onHyakoReady,
		uploadFiles,
		createDirectory,
		moveItemToDirectory,
	};
}
