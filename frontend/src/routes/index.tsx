import { createFileRoute } from "@tanstack/react-router";
import Renderer from "#/components/renderer/Renderer";
export const Route = createFileRoute("/")({ component: App });

function App() {
  return (
    <main className="min-h-screen min-w-screen">
      <Renderer />
    </main>
  );
}
