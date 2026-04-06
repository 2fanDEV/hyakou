import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "#/components/ui/card";
import type { CameraDO } from "@wasm/hyako_wasm_bindings";

export interface CameraProps {
  camera?: CameraDO | null;
}

export default function Camera({ camera }: CameraProps) {
  console.log(camera);
  return (
    <Card className="w-full h-full">
      <CardHeader>
        <CardTitle> Camera </CardTitle>
        <CardDescription>Id: {camera?.get_camera_id.get_value}</CardDescription>
      </CardHeader>
    </Card>
  );
}
