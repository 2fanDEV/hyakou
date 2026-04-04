import { useCallback, useRef } from "react";
import type { Size } from "./types/geometry";

export type RendererProps = {
  onMount: (canvas: HTMLCanvasElement) => void;
  onResize: (width: number, height: number) => void;
};

export default function RendererCanvas({ onMount, onResize }: RendererProps) {
  const startedRef = useRef<boolean>(false);
  const canvasRef = useCallback(
    (canvas: HTMLCanvasElement) => {
      let frame = 0;
      const observedSize = () => {
        const rect = canvas.getBoundingClientRect();
        const cssWidth = rect.width;
        const cssHeight = rect.height;
        if (cssHeight === 0 || cssWidth === 0) return;

        const dpr = window.devicePixelRatio || 1;
        const physicalWidth = Math.round(cssWidth * dpr);
        const physicalHeight = Math.round(cssHeight * dpr);

        canvas.width = physicalWidth;
        canvas.height = physicalHeight;

        const size: Size = {
          width: cssWidth,
          height: cssHeight,
        };

        if (!startedRef.current) {
          onMount(canvas);
          startedRef.current = true;
        }

        onResize(physicalWidth, physicalHeight);
        return size;
      };

      const observer = new ResizeObserver(() => {
        cancelAnimationFrame(frame);
        frame = requestAnimationFrame(observedSize);
      });

      observer.observe(canvas);
      frame = requestAnimationFrame(observedSize);

      return () => {
        observer.disconnect();
        cancelAnimationFrame(frame);
      };
    },
    [onMount, onResize],
  );

  return <canvas ref={canvasRef} className="block h-full w-full" />;
}
