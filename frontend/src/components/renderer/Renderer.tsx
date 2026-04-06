import type { CameraDO, Hyako } from "@wasm/hyako_wasm_bindings";
import { useEffect, useRef, useState } from "react";
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

  useWasm((hyako: Hyako) => {
    hyakoRef.current = hyako;
    setHyako(hyako);
  }, canvasState);

  useEffect(() => {
    if (!hyako) return;

    setCamera(hyako.get_camera());
  }, [hyako]);

  return (
    <div>
      <div className="flex fixed z-1 w-xs h-xl m-10 shadow-black shadow-(--shadow-left)">
        {camera ? (
          <Camera camera={camera} />
        ) : (
          <div>Renderer not initialized yet</div>
        )}
      </div>
      <div className="relative h-full w-full">
        <RendererCanvas
          onMount={setCanvasState}
          onResize={(width, height) => hyakoRef.current?.resize(width, height)}
        />
      </div>
    </div>
  );
}
