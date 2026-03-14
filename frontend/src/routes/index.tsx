import { createFileRoute } from "@tanstack/react-router";
import { useEffect } from "react";
import init from "hyako_wasm_bindings";
import { full_start, start } from "hyako_wasm_bindings";
import wasm_url from "hyako_wasm_bindings/hyako_wasm_bindings_bg.wasm?url";

export const Route = createFileRoute("/")({ component: App });

function App() {
  useEffect(() => {
    let cancelled = false;
    (async () => {
      let x = await init({ wasm_url });
      try {
        start();
      } catch (e) {
        console.error(e);
      }
      // full_start(wasm_url
      if (cancelled) return;
      console.log("Wasm initialized");
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="page-wrap px-4 pb-8 pt-14">
      <div> Hi </div>
      <div id="app"> Hyakou initialized </div>
      <div> Hi </div>
    </main>
  );
}
