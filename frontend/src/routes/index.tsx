import RendererCanvas from "#/components/renderer/canvas";
import Renderer from "#/components/renderer/Renderer";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
export const Route = createFileRoute("/")({ component: App });

function App() {
  return (
    <main>
      <Renderer />
    </main>
  );
}
