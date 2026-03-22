import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import init, { AssetInformation, Hyako } from "@wasm/hyako_wasm_bindings.js";
import wasm_url from "@wasm/hyako_wasm_bindings_bg.wasm?url";

export const Route = createFileRoute("/")({ component: App });

function App() {
  let canvasRef = useRef<HTMLCanvasElement>(null);
  let hyakoRef = useRef<Hyako | null>(null);
  let [wasmReady, setWasmReady] = useState(false);
  let [selectedFile, setSelectedFile] = useState<File | null>(null);
  let [statusMessage, setStatusMessage] = useState<string>("");

  useEffect(() => {
    let cancelled = false;
    (async () => {
      await init({ module_or_path: wasm_url });
      try {
        if (canvasRef.current) {
          let hyako = new Hyako(canvasRef.current);
          hyako.start_rendering();
          hyakoRef.current = hyako;
          if (!cancelled) {
            setWasmReady(true);
            setStatusMessage("Renderer initialized");
          }
        }
      } catch (e) {
        console.error(e);
        if (!cancelled) {
          setStatusMessage("Renderer failed to initialize");
        }
      }
      if (cancelled) return;
      console.log("Wasm initialized");
    })();

    return () => {
      cancelled = true;
      hyakoRef.current = null;
    };
  }, []);

  let onUploadClicked = async () => {
    if (!selectedFile) {
      setStatusMessage("Select a file before uploading");
      return;
    }

    if (hyakoRef.current === null) {
      setStatusMessage("Renderer is not ready yet");
      return;
    }

    try {
      let bytes = new Uint8Array(await selectedFile.arrayBuffer());
      let modifiedAtSeconds = Math.floor(selectedFile.lastModified / 1000);
      let assetInformation = new AssetInformation(
        bytes,
        selectedFile.name,
        BigInt(selectedFile.size),
        modifiedAtSeconds,
      );
      hyakoRef.current.upload_file(assetInformation);
      setStatusMessage(`Uploaded ${selectedFile.name}`);
    } catch (uploadError) {
      console.error(uploadError);
      setStatusMessage("Upload failed");
    }
  };

  return (
    <main className="page-wrap px-4 pb-8 pt-14">
      <div id="app">Hyakou initialized</div>
      <div className="mb-3 mt-3 flex flex-wrap items-center gap-2">
        <input
          type="file"
          onChange={(event) => setSelectedFile(event.target.files?.[0] ?? null)}
        />
        <button
          type="button"
          onClick={onUploadClicked}
          disabled={!wasmReady || selectedFile === null}
        >
          Upload file
        </button>
      </div>
      <div>{statusMessage}</div>
      <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />
    </main>
  );
}
