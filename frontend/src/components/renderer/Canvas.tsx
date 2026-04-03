import { useCallback } from "react";

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
      <canvas ref={canvasRef} className="p-10  w-full h-full"></canvas>
    </div>
  );
}
