import type { CameraDO, Hyako } from "@wasm/hyako_wasm_bindings";
import { useEffect, useRef, useState } from "react";
import FileDirectory from "./asset/fileDirectory";
import ItemDropZone from "./asset/fileDroppable";
import useFileDrop from "./asset/useFileDrop";
import RendererCanvas from "./Canvas";
import Camera from "./camera/camera";
import useWasm from "./hooks/wasm";

export default function Renderer() {
	const hyakoRef = useRef<Hyako | null>(null);
	const [hyako, setHyako] = useState<Hyako | null>(null);
	const [canvasState, setCanvasState] = useState<HTMLCanvasElement | null>(
		null,
	);
	const [camera, setCamera] = useState<CameraDO | null>(null);
	const { state, onHyakoReady, uploadFiles, createDirectory, moveItems } =
		useFileDrop();

	useWasm((instance: Hyako) => {
		hyakoRef.current = instance;
		setHyako(instance);
		onHyakoReady(instance);
	}, canvasState);

	useEffect(() => {
		if (!hyako) return;
		setCamera(hyako.get_camera());
	}, [hyako]);

	return (
		<div>
			<FileDirectory
				state={state}
				onCreateDirectory={createDirectory}
				onMoveItems={moveItems}
			/>
			<div className="fixed left-4 top-4 z-10 sm:left-10 sm:top-10">
				<Camera key={camera?.get_camera_id.get_value} camera={camera} />
			</div>
			<div className="relative h-full w-full">
				<ItemDropZone
					onFileDrop={(files) => {
						if (hyakoRef.current) {
							uploadFiles(hyakoRef.current, files);
						}
					}}
				>
					<RendererCanvas
						onMount={setCanvasState}
						onResize={(width, height) =>
							hyakoRef.current?.resize(width, height)
						}
					/>
				</ItemDropZone>
			</div>
		</div>
	);
}
