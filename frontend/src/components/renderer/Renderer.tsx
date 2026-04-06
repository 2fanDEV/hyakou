import {
  type CameraDO,
  Coordinates3,
  type Hyako,
} from "@wasm/hyako_wasm_bindings";
import { useEffect, useRef, useState } from "react";
import RendererCanvas from "./Canvas";
import Camera, { CameraSkeleton } from "./camera/camera";
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

  const setEyePosition = (coords: { x: number; y: number; z: number }) => {
    const renderer = hyakoRef.current;
    if (!renderer) return;
    renderer.set_coords(new Coordinates3(coords.x, coords.y, coords.z));
    setCamera(renderer.get_camera());
  };

  return (
    <div>
      <div className="fixed left-4 top-4 z-10 sm:left-10 sm:top-10">
        <Camera
          key={camera?.get_camera_id.get_value}
          camera={camera}
          onSetEye={setEyePosition}
        />
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
