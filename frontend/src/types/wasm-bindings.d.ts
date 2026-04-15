declare module "@wasm/*.js" {
  export function start(canvas: HTMLCanvasElement): void;

  export default function init(
    input?: string | URL | Request | { module_or_path: string | URL | Request },
  ): Promise<unknown>;
}

declare module "@wasm/*.wasm?url" {
  const wasmUrl: string;

  export default wasmUrl;
}
