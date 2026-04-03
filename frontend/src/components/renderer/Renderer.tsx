import { Hyako } from "@wasm/hyako_wasm_bindings";
import { useRef, useState } from "react";
import RendererCanvas from "./Canvas";
import useWasm from "./hooks/wasm";

export default function Renderer() {
  const hyakoRef = useRef<Hyako | null>(null);
  const [canvasState, setCanvasState] = useState<HTMLCanvasElement | null>();
  useWasm((ref: Hyako) => (hyakoRef.current = ref), canvasState);

  return (
    <div>
      <RendererCanvas onMount={setCanvasState} />
    </div>
  );
}
