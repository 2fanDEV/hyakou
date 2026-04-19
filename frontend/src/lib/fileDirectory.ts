export type UploadStatus = "pending" | "success" | "done" | "error";

export type FileDirectoryItem = {
	id: string;
	fileName: string;
	status: UploadStatus;
	errorMessage?: string;
	directoryId?: string;
};

export type FileDirectory = {
	id: string;
	name: string;
};

export type FileDirectoryState = {
	items: FileDirectoryItem[];
	directories: FileDirectory[];
};
