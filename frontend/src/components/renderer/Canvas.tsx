import { useMutation } from "@tanstack/react-query";
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
        const size: Size = {
          width: rect.width,
          height: rect.height,
        };
        if (size.height === 0 || size.width === 0) return;
        if (!startedRef.current) {
          onMount(canvas);
          startedRef.current = true;
        }
        onResize(size.width, size.height);
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

  return <canvas ref={canvasRef} className="p-10 w-full h-full" />;
}
