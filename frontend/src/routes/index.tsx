import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef } from "react";
import init, { start } from "@wasm/hyako_wasm_bindings.js";
import wasm_url from "@wasm/hyako_wasm_bindings_bg.wasm?url";

export const Route = createFileRoute("/")({ component: App });

function App() {
  let canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      await init({ module_or_path: wasm_url });
      try {
        if (canvasRef.current) {
          start(canvasRef.current);
        }
      } catch (e) {
        console.error(e);
      }
      if (cancelled) return;
      console.log("Wasm initialized");
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="page-wrap px-4 pb-8 pt-14">
      <div id="app"> Hyakou initialized </div>
      <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />
    </main>
  );
}
