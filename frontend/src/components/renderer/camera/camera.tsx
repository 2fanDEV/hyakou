import type { CameraDO } from "@wasm/hyako_wasm_bindings";
import { ChevronDownIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from "#/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "#/components/ui/collapsible";
import { Field, FieldTitle } from "#/components/ui/field";
import { Skeleton } from "#/components/ui/skeleton";

function fmt(value: number): string {
  return value.toFixed(4);
}

function VectorField({
  label,
  x,
  y,
  z,
}: {
  label: string;
  x: number;
  y: number;
  z: number;
}) {
  return (
    <Field>
      <FieldTitle>{label}</FieldTitle>
      <div className="grid grid-cols-3 gap-1.5 text-[10px] tabular-nums">
        <span><span className="text-foreground font-medium">X</span> <span className="tabular-nums text-muted-foreground">{fmt(x)}</span></span>
        <span><span className="text-foreground font-medium">Y</span> <span className="tabular-nums text-muted-foreground">{fmt(y)}</span></span>
        <span><span className="text-foreground font-medium">Z</span> <span className="tabular-nums text-muted-foreground">{fmt(z)}</span></span>
      </div>
    </Field>
  );
}

function ScalarRow({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center justify-between gap-1">
      <span className="text-[10px] text-muted-foreground">{label}</span>
      <span className="text-[10px] tabular-nums">{value}</span>
    </div>
  );
}

export interface CameraProps {
  camera?: CameraDO | null;
}

export default function Camera({ camera }: CameraProps) {
  const [isOpen, setIsOpen] = useState(true);

  if (!camera) return <CameraSkeleton />;

  return (
    <Card size="sm" className="w-64">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CardHeader>
          <CollapsibleTrigger asChild>
            <button type="button" className="flex min-w-0 cursor-pointer flex-col text-left">
              <CardTitle>CAMERA</CardTitle>
              <span className="text-[10px] text-muted-foreground">
                {camera.get_camera_id.get_value}
              </span>
            </button>
          </CollapsibleTrigger>
          <CardAction>
            <CollapsibleTrigger asChild>
              <Button variant="ghost" size="icon-xs">
                <ChevronDownIcon
                  className={`size-3.5 transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
                />
              </Button>
            </CollapsibleTrigger>
          </CardAction>
        </CardHeader>

        <CollapsibleContent forceMount>
          <div
            className={`grid transition-all duration-200 ease-in-out ${isOpen ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0"}`}
          >
            <div className="overflow-hidden">
              <CardContent className="flex flex-col gap-3">
                <VectorField label="Eye" x={camera.eye.x} y={camera.eye.y} z={camera.eye.z} />
                <VectorField label="Up" x={camera.up.x} y={camera.up.y} z={camera.up.z} />
                <VectorField label="Target" x={camera.target.x} y={camera.target.y} z={camera.target.z} />

                <div className="h-px bg-foreground/10" />

                <div className="grid grid-cols-2 gap-x-3 gap-y-1">
                  <ScalarRow label="Fovy" value={fmt(camera.fovy)} />
                  <ScalarRow label="Aspect" value={fmt(camera.aspect)} />
                  <ScalarRow label="Near" value={fmt(camera.znear)} />
                  <ScalarRow label="Far" value={fmt(camera.zfar)} />
                </div>

                <div className="h-px bg-foreground/10" />

                <div className="grid grid-cols-2 gap-x-3 gap-y-1">
                  <ScalarRow label="Speed" value={fmt(camera.speed)} />
                  <ScalarRow label="Sensitivity" value={fmt(camera.sensitivity)} />
                  <ScalarRow label="Smoothing" value={fmt(camera.smoothing_factor)} />
                </div>
              </CardContent>
            </div>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </Card>
  );
}

export function CameraSkeleton() {
  return (
    <Card size="sm" className="w-64">
      <CardHeader>
        <div className="flex min-w-0 flex-col gap-1">
          <Skeleton className="h-4 w-16" />
          <Skeleton className="h-3 w-40" />
        </div>
        <CardAction>
          <Skeleton className="size-5 rounded-sm" />
        </CardAction>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <Field>
          <Skeleton className="h-3 w-8" />
          <div className="grid grid-cols-3 gap-1.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
          </div>
        </Field>
        <Field>
          <Skeleton className="h-3 w-7" />
          <div className="grid grid-cols-3 gap-1.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
          </div>
        </Field>
        <Field>
          <Skeleton className="h-3 w-10" />
          <div className="grid grid-cols-3 gap-1.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-full" />
          </div>
        </Field>

        <div className="h-px bg-foreground/10" />

        <div className="grid grid-cols-2 gap-x-3 gap-y-1">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
        </div>

        <div className="h-px bg-foreground/10" />

        <div className="grid grid-cols-2 gap-x-3 gap-y-1">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
        </div>
      </CardContent>
    </Card>
  );
}
