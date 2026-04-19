import { CheckIcon, XIcon } from "@phosphor-icons/react";
import type { FileDirectoryItem as FileDirectoryItemType } from "#/lib/fileDirectory";
import { getFileIcon } from "#/lib/fileIcon";
import { cn } from "#/lib/utils";

type Props = {
	item: FileDirectoryItemType;
	onDragStart?: (itemId: string) => void;
};

function StatusSlot({ status }: { status: FileDirectoryItemType["status"] }) {
	const isSettled = status === "success" || status === "error";
	const StatusIcon =
		status === "success" ? CheckIcon : status === "error" ? XIcon : null;

	return (
		<span className="upload-status-slot" aria-hidden="true">
			{(status === "pending" || isSettled) && (
				<span
					className={cn(
						"upload-progress-track",
						isSettled && "upload-progress-track-exit",
					)}
				>
					<span className="upload-progress-bar" />
				</span>
			)}
			{StatusIcon && (
				<span className="upload-status-pop">
					<StatusIcon
						className={cn(
							"upload-status-symbol",
							status === "error" ? "text-destructive" : "text-primary",
						)}
						weight="bold"
						size={14}
					/>
				</span>
			)}
		</span>
	);
}

export default function FileDirectoryItem({ item, onDragStart }: Props) {
	const FileIcon = getFileIcon(item.fileName);
	const handleDragStart = onDragStart
		? (event: React.DragEvent) => {
				event.dataTransfer.setData("text/plain", item.id);
				event.dataTransfer.effectAllowed = "move";
				onDragStart(item.id);
			}
		: undefined;

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: intentional drag source
		<div
			className={cn(
				"upload-item",
				item.status === "error" && "text-destructive",
			)}
			title={item.status === "error" ? item.errorMessage : item.fileName}
			draggable={!!onDragStart}
			onDragStart={handleDragStart}
		>
			<span className="upload-file-icon" aria-hidden="true">
				<FileIcon size={14} weight="regular" />
			</span>
			<span className="min-w-0 flex-1 truncate">{item.fileName}</span>
			<StatusSlot status={item.status} />
		</div>
	);
}
