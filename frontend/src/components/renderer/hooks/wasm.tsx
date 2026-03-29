import { useQuery } from "@tanstack/react-query";
import init, { Hyako } from "@wasm/hyako_wasm_bindings";
import wasm_url from "@wasm/hyako_wasm_bindings_bg.wasm?url";

export default function useWasm(
  onInit: (r: Hyako) => void,
  canvasState: HTMLCanvasElement | null | undefined,
) {
  useQuery({
    queryKey: ["hyako-wasm-init"],
    queryFn: async () => {
      await init({ module_or_path: wasm_url });
      if (!canvasState) return;
      const hyako = new Hyako(canvasState);
      hyako.start_rendering();
      onInit(hyako);
      return hyako;
    },
    enabled: canvasState != null,
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
  });
}
