import {
	type DragEvent,
	type DragEventHandler,
	type ReactNode,
	useState,
} from "react";
import invariant from "tiny-invariant";

type DropZoneProps = {
	children: ReactNode;
	onFileDrop: (files: FileList) => void;
};

export default function ItemDropZone({ children, onFileDrop }: DropZoneProps) {
	const [externalHover, setExternalHover] = useState(false);

	const handleDragEnter: DragEventHandler<HTMLDivElement> = (
		event: DragEvent<HTMLDivElement>,
	) => {
		invariant(event.dataTransfer, "event.dataTransfer is undefined");
		console.log(event.dataTransfer);
		[...event.dataTransfer.items].some((item) => item.kind === "file")
			? setExternalHover(true)
			: setExternalHover(false);
	};

	const handleDragLeave = () => {
		setExternalHover(false);
	};

	const handleDrop = (event: DragEvent<HTMLDivElement>) => {
		event.preventDefault();
		onFileDrop(event.dataTransfer.files);
		setExternalHover(false);
	};

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: It is a dropzone and every other element seemed unnecessary
		<div
			className="h-full w-full"
			onDragEnter={handleDragEnter}
			onDragOver={(event) => event.preventDefault()}
			onDragLeave={handleDragLeave}
			onDrop={handleDrop}
		>
			{children}
		</div>
	);
}
