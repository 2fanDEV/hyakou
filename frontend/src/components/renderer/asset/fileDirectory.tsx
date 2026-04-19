import { PlusIcon } from "@phosphor-icons/react";
import { ContextMenu as ContextMenuPrimitive } from "radix-ui";
import { useRef, useState } from "react";
import { DraggablePanel } from "#/components/ui/DraggablePanel";
import type { FileDirectoryState } from "#/lib/fileDirectory";
import FileDirectoryFolder from "./FileDirectoryFolder";
import FileDirectoryItem from "./FileDirectoryItem";

type Props = {
	state: FileDirectoryState;
	onCreateDirectory: (name: string) => void;
	onMoveItem: (itemId: string, directoryId: string | undefined) => void;
};

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
		<div className="flex items-center gap-1 px-2.5 py-1.5">
			<input
				ref={inputRef}
				type="text"
				value={value}
				onChange={(e) => setValue(e.target.value)}
				onKeyDown={(e) => {
					if (e.key === "Enter" && value.trim()) {
						onConfirm(value.trim());
					} else if (e.key === "Escape") {
						onCancel();
					}
				}}
				onBlur={() => {
					if (value.trim()) onConfirm(value.trim());
					else onCancel();
				}}
				placeholder="Directory name…"
				className="min-w-0 flex-1 rounded border border-border bg-background px-1.5 py-0.5 text-xs outline-none focus:border-ring"
			/>
		</div>
	);
}

export default function FileDirectory({
	state,
	onCreateDirectory,
	onMoveItem,
}: Props) {
	const [addingDirectory, setAddingDirectory] = useState(false);

	const handleConfirmDirectory = (name: string) => {
		onCreateDirectory(name);
		setAddingDirectory(false);
	};

	const ungroupedItems = state.items.filter((item) => !item.directoryId);

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

	return (
		<ContextMenuPrimitive.Root>
			<ContextMenuPrimitive.Trigger asChild>
				<div>
					<DraggablePanel title="Files" headerActions={addDirectoryButton}>
						{ungroupedItems.map((item) => (
							<FileDirectoryItem
								key={item.id}
								item={item}
								onDragStart={
									state.directories.length > 0 ? () => {} : undefined
								}
							/>
						))}

						{state.directories.map((dir) => (
							<FileDirectoryFolder
								key={dir.id}
								directory={dir}
								items={state.items.filter(
									(item) => item.directoryId === dir.id,
								)}
								onMoveItem={onMoveItem}
							/>
						))}

						{addingDirectory && (
							<NewDirectoryInput
								onConfirm={handleConfirmDirectory}
								onCancel={() => setAddingDirectory(false)}
							/>
						)}

						{state.items.length === 0 &&
							state.directories.length === 0 &&
							!addingDirectory && (
								<p className="px-3 py-4 text-center text-xs text-muted-foreground/50 italic">
									Drop files anywhere to load them
								</p>
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
