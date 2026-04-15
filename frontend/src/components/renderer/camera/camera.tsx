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
        <span>
          <span className="text-foreground font-medium">X</span>{" "}
          <span className="tabular-nums text-muted-foreground">{fmt(x)}</span>
        </span>
        <span>
          <span className="text-foreground font-medium">Y</span>{" "}
          <span className="tabular-nums text-muted-foreground">{fmt(y)}</span>
        </span>
        <span>
          <span className="text-foreground font-medium">Z</span>{" "}
          <span className="tabular-nums text-muted-foreground">{fmt(z)}</span>
        </span>
      </div>
    </Field>
  );
}

function ScalarRow({ label, value }: { label: string; value: string }) {
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

export default function Camera({ camera }: CameraProps) {}
