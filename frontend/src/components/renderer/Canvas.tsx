import { useQuery } from "@tanstack/react-query";
import init, { Hyako } from "@wasm/hyako_wasm_bindings";
import wasm_url from "@wasm/hyako_wasm_bindings_bg.wasm?url";
import { useCallback, useRef, useState, type RefObject } from "react";

export type RendererProps = {
  onMount: (canvas: HTMLCanvasElement) => void;
};

export default function RendererCanvas({ onMount }: RendererProps) {
  const canvasRef = useCallback(
    (element: HTMLCanvasElement | null) => {
      if (element) onMount(element);
    },
    [onMount],
  );

  return (
    <div>
      <canvas ref={canvasRef} className="p-10 w-full h-full"></canvas>
    </div>
  );
}
