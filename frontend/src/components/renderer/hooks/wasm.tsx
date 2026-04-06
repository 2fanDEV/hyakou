import { useQuery } from "@tanstack/react-query";
import init, { Hyako } from "@wasm/hyako_wasm_bindings";
import wasm_url from "@wasm/hyako_wasm_bindings_bg.wasm?url";
import { useRef } from "react";

export default function useWasm(
  onInit: (r: Hyako) => void,
  canvasState: HTMLCanvasElement | null | undefined,
) {
  const canvasRef = useRef(canvasState);
  canvasRef.current = canvasState;
  const onInitRef = useRef(onInit);
  onInitRef.current = onInit;

  useQuery({
    queryKey: ["hyako-wasm-init"],
    queryFn: async () => {
      await init({ module_or_path: wasm_url });
      if (!canvasRef.current) return;
      const hyako = new Hyako(canvasRef.current);
      hyako.start_rendering();
      await waitUntilRendererReady(hyako);
      onInitRef.current(hyako);
      return hyako;
    },
    enabled: canvasRef.current != null,
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
  });
}

const waitUntilRendererReady = async (hyako: Hyako) => {
  while (!hyako.is_renderer_ready()) {
    await new Promise<void>((resolve) => window.setTimeout(resolve, 2));
    console.log("LOL", hyako.is_renderer_ready());
  }
  return true;
};
