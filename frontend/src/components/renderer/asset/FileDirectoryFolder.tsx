import { CaretRightIcon, FolderIcon } from "@phosphor-icons/react";
import { useState } from "react";
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from "#/components/ui/collapsible";
import type {
	FileDirectory,
	FileDirectoryItem as FileDirectoryItemType,
} from "#/lib/fileDirectory";
import FileDirectoryItem from "./FileDirectoryItem";

type Props = {
	directory: FileDirectory;
	items: FileDirectoryItemType[];
	onMoveItem: (itemId: string, directoryId: string) => void;
};

export default function FileDirectoryFolder({
	directory,
	items,
	onMoveItem,
}: Props) {
	const [open, setOpen] = useState(true);
	const [dropOver, setDropOver] = useState(false);

	const handleDragOver = (e: React.DragEvent) => {
		e.preventDefault();
		e.dataTransfer.dropEffect = "move";
		setDropOver(true);
	};

	const handleDragLeave = () => setDropOver(false);

	const handleDrop = (e: React.DragEvent) => {
		e.preventDefault();
		setDropOver(false);
		const itemId = e.dataTransfer.getData("text/plain");
		if (itemId) onMoveItem(itemId, directory.id);
	};

	return (
		<Collapsible open={open} onOpenChange={setOpen}>
			<CollapsibleTrigger
				className={`flex w-full items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium transition-colors ${dropOver ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:text-foreground"}`}
				onDragOver={handleDragOver}
				onDragLeave={handleDragLeave}
				onDrop={handleDrop}
			>
				<CaretRightIcon
					size={10}
					aria-hidden="true"
					className="shrink-0 transition-transform duration-150"
					style={{ transform: open ? "rotate(90deg)" : undefined }}
				/>
				<FolderIcon size={13} aria-hidden="true" weight="regular" />
				<span className="truncate">{directory.name}</span>
				<span className="ml-auto shrink-0 tabular-nums text-muted-foreground/60">
					{items.length}
				</span>
			</CollapsibleTrigger>
			<CollapsibleContent className="animate-slide-down overflow-hidden data-[state=closed]:animate-slide-up">
				{items.map((item) => (
					<div key={item.id} className="pl-3">
						<FileDirectoryItem item={item} onDragStart={() => {}} />
					</div>
				))}
				{items.length === 0 && (
					<p className="px-5 py-2 text-xs text-muted-foreground/50 italic">
						Empty
					</p>
				)}
			</CollapsibleContent>
		</Collapsible>
	);
}
