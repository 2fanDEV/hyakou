import {
	CaretDownIcon,
	CaretRightIcon,
	CheckIcon,
	FolderIcon,
	type Icon,
	PlusIcon,
	XIcon,
} from "@phosphor-icons/react";
import { ContextMenu as ContextMenuPrimitive } from "radix-ui";
import { useRef, useState } from "react";
import {
	type MoveHandler,
	type NodeRendererProps,
	type RowRendererProps,
	Tree,
} from "react-arborist";
import { DraggablePanel } from "#/components/ui/DraggablePanel";
import type {
	FileDirectoryNode,
	FileDirectoryState,
} from "#/lib/fileDirectory";
import { getFileIcon } from "#/lib/fileIcon";
import { cn } from "#/lib/utils";

type Props = {
	state: FileDirectoryState;
	onCreateDirectory: (name: string) => void;
	onMoveItems: (
		itemIds: string[],
		directoryId: string | null,
		index: number,
	) => void;
};

const PANEL_WIDTH = 280;
const PANEL_HEIGHT = 320;
const TREE_ROW_HEIGHT = 28;
const TREE_INDENT = 8;

function StatusSlot({
	status,
}: {
	status: Extract<FileDirectoryNode, { kind: "file" }>["status"];
}) {
	if (status === "done") {
		return <span className="upload-status-slot" aria-hidden="true" />;
	}

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
							status === "error"
								? "text-[var(--color-upload-error)]"
								: "text-[var(--color-upload-success)]",
						)}
						weight="bold"
						size={14}
					/>
				</span>
			)}
		</span>
	);
}

function ExplorerNode({
	node,
	style,
	dragHandle,
}: NodeRendererProps<FileDirectoryNode>) {
	const data = node.data;
	const isFolder = data.kind === "folder";
	const FileIcon = !isFolder ? (getFileIcon(data.name) as Icon) : undefined;

	return (
		<div style={style} className="px-1.5">
			<div
				role="treeitem"
				tabIndex={-1}
				ref={dragHandle}
				className={cn(
					"upload-item rounded-md",
					node.isSelected && "bg-accent text-accent-foreground",
					node.willReceiveDrop && "bg-accent/70",
					!isFolder && data.status === "error" && "text-destructive",
				)}
				onClick={node.handleClick}
				onKeyDown={(event) => {
					if (event.key === "Enter" || event.key === " ") {
						event.preventDefault();
						node.handleClick(event as never);
					}
				}}
				title={
					!isFolder && data.status === "error" ? data.errorMessage : data.name
				}
			>
				{isFolder ? (
					<>
						<button
							type="button"
							className="upload-file-icon rounded hover:bg-muted"
							onClick={(event) => {
								event.stopPropagation();
								node.toggle();
							}}
							aria-label={node.isOpen ? "Collapse folder" : "Expand folder"}
						>
							{node.isOpen ? (
								<CaretDownIcon size={12} weight="bold" />
							) : (
								<CaretRightIcon size={12} weight="bold" />
							)}
						</button>
						<span className="upload-file-icon" aria-hidden="true">
							<FolderIcon size={14} weight="regular" />
						</span>
					</>
				) : (
					<>
						<span className="upload-file-icon" aria-hidden="true">
							{FileIcon ? <FileIcon size={14} weight="regular" /> : null}
						</span>
						<span className="upload-file-spacer" aria-hidden="true" />
					</>
				)}
				<span className="min-w-0 flex-1 truncate">{data.name}</span>
				{!isFolder && <StatusSlot status={data.status} />}
			</div>
		</div>
	);
}

function NewDirectoryInput({
	onConfirm,
	onCancel,
}: {
	onConfirm: (name: string) => void;
	onCancel: () => void;
}) {
	const [value, setValue] = useState("");
	const inputRef = useRef<HTMLInputElement>(null);

	return (
		<div className="px-2.5 py-1.5">
			<input
				ref={inputRef}
				type="text"
				value={value}
				onChange={(event) => setValue(event.target.value)}
				onKeyDown={(event) => {
					if (event.key === "Enter" && value.trim()) {
						onConfirm(value.trim());
					} else if (event.key === "Escape") {
						onCancel();
					}
				}}
				onBlur={() => {
					if (value.trim()) onConfirm(value.trim());
					else onCancel();
				}}
				placeholder="Directory name..."
				className="w-full rounded border border-border bg-background px-2 py-1 text-xs outline-none focus:border-ring"
			/>
		</div>
	);
}

function TreeRow({
	innerRef,
	attrs,
	children,
}: RowRendererProps<FileDirectoryNode>) {
	return (
		<div ref={innerRef} {...attrs}>
			{children}
		</div>
	);
}

export default function FileDirectory({
	state,
	onCreateDirectory,
	onMoveItems,
}: Props) {
	const [addingDirectory, setAddingDirectory] = useState(false);

	const handleConfirmDirectory = (name: string) => {
		onCreateDirectory(name);
		setAddingDirectory(false);
	};

	const addDirectoryButton = (
		<button
			type="button"
			onClick={() => setAddingDirectory(true)}
			className="flex h-5 w-5 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
			aria-label="New directory"
		>
			<PlusIcon size={10} weight="bold" aria-hidden="true" />
		</button>
	);

	const treeHeight = Math.max(
		TREE_ROW_HEIGHT,
		PANEL_HEIGHT - (addingDirectory ? TREE_ROW_HEIGHT + 10 : 0),
	);

	const handleMove: MoveHandler<FileDirectoryNode> = ({
		dragIds,
		parentId,
		index,
	}) => {
		onMoveItems(dragIds, parentId, index);
	};

	const emptyState = state.nodes.length === 0 && !addingDirectory;

	return (
		<ContextMenuPrimitive.Root>
			<ContextMenuPrimitive.Trigger asChild>
				<div>
					<DraggablePanel
						title="Explorer"
						headerActions={addDirectoryButton}
						defaultWidth={PANEL_WIDTH}
						defaultHeight={PANEL_HEIGHT}
					>
						{emptyState ? (
							<p className="px-3 py-4 text-center text-xs text-muted-foreground/50 italic">
								Drop files anywhere to load them
							</p>
						) : (
							<Tree<FileDirectoryNode>
								data={state.nodes}
								width="100%"
								height={treeHeight}
								rowHeight={TREE_ROW_HEIGHT}
								indent={TREE_INDENT}
								padding={4}
								openByDefault={true}
								disableMultiSelection
								disableEdit
								childrenAccessor={(node) =>
									node.kind === "folder" ? node.children : null
								}
								idAccessor="id"
								onMove={handleMove}
								renderRow={TreeRow}
							>
								{ExplorerNode}
							</Tree>
						)}

						{addingDirectory && (
							<NewDirectoryInput
								onConfirm={handleConfirmDirectory}
								onCancel={() => setAddingDirectory(false)}
							/>
						)}
					</DraggablePanel>
				</div>
			</ContextMenuPrimitive.Trigger>

			<ContextMenuPrimitive.Portal>
				<ContextMenuPrimitive.Content className="z-50 min-w-[120px] overflow-hidden rounded-md border border-border bg-popover p-1 shadow-md">
					<ContextMenuPrimitive.Item
						className="flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-xs outline-none hover:bg-accent hover:text-accent-foreground"
						onSelect={() => setAddingDirectory(true)}
					>
						<PlusIcon size={10} weight="bold" aria-hidden="true" />
						New Directory
					</ContextMenuPrimitive.Item>
				</ContextMenuPrimitive.Content>
			</ContextMenuPrimitive.Portal>
		</ContextMenuPrimitive.Root>
	);
}
