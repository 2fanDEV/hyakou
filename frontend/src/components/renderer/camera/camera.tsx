import type { CameraDO } from "@wasm/hyako_wasm_bindings";
import { Field, FieldTitle } from "#/components/ui/field";

export interface CameraProps {
  camera?: CameraDO | null;
}

export default function Camera({ camera }: CameraProps) {
  return <div></div>;
}
