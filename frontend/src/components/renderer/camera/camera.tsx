import type { CameraDO } from "@wasm/hyako_wasm_bindings";
import { ChevronDownIcon, MinusIcon, PlusIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "#/components/ui/collapsible";
import { Field, FieldTitle } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Skeleton } from "#/components/ui/skeleton";
import React from "react";

export interface CameraProps {
  camera?: CameraDO | null;
  onSetEye: (coords: { x: number; y: number; z: number }) => void;
}

type EyeDraft = { x: string; y: string; z: string };

function fmt(v: number | undefined, d = 3) {
  return typeof v === "number" ? v.toFixed(d) : "-";
}

function fmtVec(v: { x: number; y: number; z: number } | undefined, d = 3) {
  if (!v) return "-";
  return `${v.x.toFixed(d)}, ${v.y.toFixed(d)}, ${v.z.toFixed(d)}`;
}

function draftFrom(camera: CameraDO | null | undefined): EyeDraft {
  return {
    x: camera?.eye.x.toString() ?? "",
    y: camera?.eye.y.toString() ?? "",
    z: camera?.eye.z.toString() ?? "",
  };
}

function isDirty(camera: CameraDO | null | undefined, draft: EyeDraft) {
  if (!camera) return false;
  return (
    draft.x !== camera.eye.x.toString() ||
    draft.y !== camera.eye.y.toString() ||
    draft.z !== camera.eye.z.toString()
  );
}

function isValidDraft(draft: EyeDraft) {
  return (
    draft.x.trim() !== "" &&
    draft.y.trim() !== "" &&
    draft.z.trim() !== "" &&
    !Number.isNaN(Number(draft.x)) &&
    !Number.isNaN(Number(draft.y)) &&
    !Number.isNaN(Number(draft.z))
  );
}

const AXES = ["x", "y", "z"] as const;

export default function Camera({ camera, onSetEye }: CameraProps) {
  const [cardOpen, setCardOpen] = useState(true);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [eyeDraft, setEyeDraft] = useState<EyeDraft>(() => draftFrom(camera));
  const [eyeHistory, setEyeHistory] = useState<EyeDraft[] | []>([]);

  const dirty = isDirty(camera, eyeDraft);
  const valid = isValidDraft(eyeDraft);
  const cameraId = camera?.get_camera_id.get_value ?? "-";

  const apply = () => {
    if (!valid) return;
    onSetEye({
      x: Number(eyeDraft.x),
      y: Number(eyeDraft.y),
      z: Number(eyeDraft.z),
    });
  };

  return (
    <Collapsible open={cardOpen} onOpenChange={setCardOpen}>
      <Card
        size="sm"
        className="w-[19rem] max-w-[calc(100vw-2rem)] overflow-hidden ring-2 shadow-[var(--shadow-left)]"
      >
        {camera ? (
          <CardHeader className="grid-cols-[1fr_auto] items-center pb-0">
            <CardTitle className="text-sm font-medium">Camera</CardTitle>
            <CollapsibleTrigger asChild>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label={
                  cardOpen ? "Minimize camera panel" : "Expand camera panel"
                }
              >
                {cardOpen ? <MinusIcon /> : <PlusIcon />}
              </Button>
            </CollapsibleTrigger>
          </CardHeader>
        ) : (
          <CardHeader className="grid-cols-[1fr_auto] items-center pb-0">
            <Skeleton className="h-4 w-16" />
            <Skeleton className="h-5 w-5 rounded-sm" />
          </CardHeader>
        )}
        {camera ? (
          <CollapsibleContent className="overflow-hidden data-[state=closed]:animate-slide-up data-[state=open]:animate-slide-down">
            <CardContent className="grid gap-3 pt-2">
              <Field className="gap-1">
                <FieldTitle className="text-[0.65rem] uppercase tracking-[0.12em] text-muted-foreground">
                  ID:
                  <span className="truncate font-mono text-[0.6rem] text-foreground">
                    {cameraId}
                  </span>
                </FieldTitle>
              </Field>

              <Field className="gap-1">
                <FieldTitle className="text-[0.65rem] uppercase tracking-[0.14em] text-muted-foreground">
                  Mode:
                  <span className="bg-muted max-w-auto p-1 rounded-lg text-center">
                    PAN
                  </span>
                </FieldTitle>
              </Field>

              <div className="grid gap-2">
                <div className="flex items-center justify-between">
                  <FieldTitle>Eye</FieldTitle>
                  {dirty ? (
                    <span className="text-[0.65rem] text-muted-foreground">
                      Edited
                    </span>
                  ) : null}
                </div>
                <div className="grid grid-cols-3 gap-2">
                  {AXES.map((axis) => (
                    <div key={axis} className="relative">
                      <span className="pointer-events-none absolute left-1.5 top-1/2 -translate-y-1/2 text-[0.6rem] font-medium uppercase text-muted-foreground">
                        {axis}
                      </span>
                      <Input
                        className="pl-5 text-xs tabular-nums"
                        inputMode="decimal"
                        value={eyeDraft[axis]}
                        onChange={(e) =>
                          setEyeDraft((d) => ({
                            ...d,
                            [axis]: e.target.value,
                          }))
                        }
                      />
                    </div>
                  ))}
                </div>
                <div className="flex justify-end gap-1.5">
                  <Button
                    variant="outline"
                    size="xs"
                    disabled={!camera || !dirty}
                    onClick={() => setEyeDraft(draftFrom(camera))}
                  >
                    Reset
                  </Button>
                  <Button
                    size="xs"
                    disabled={!camera || !valid}
                    onClick={apply}
                  >
                    Apply
                  </Button>
                </div>
              </div>

              <Collapsible open={detailsOpen} onOpenChange={setDetailsOpen}>
                <div className="rounded-md border border-border/70 bg-muted/20 p-2.5">
                  <CollapsibleTrigger asChild>
                    <button
                      type="button"
                      className="flex w-full items-center justify-between text-xs font-medium text-muted-foreground transition-colors hover:text-foreground"
                    >
                      <span>Details</span>
                      <ChevronDownIcon
                        className={`size-3 transition-transform ${detailsOpen ? "rotate-180" : ""}`}
                      />
                    </button>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="overflow-hidden data-[state=closed]:animate-slide-up data-[state=open]:animate-slide-down">
                    <div className="mt-2 grid gap-1 border-t border-border/60 pt-2 text-xs">
                      <DetailRow
                        label="Target"
                        value={fmtVec(camera?.target)}
                      />
                      <DetailRow label="Up" value={fmtVec(camera?.up)} />
                      <DetailRow label="FoV" value={fmt(camera?.fovy, 2)} />
                      <DetailRow label="Aspect" value={fmt(camera?.aspect)} />
                      <DetailRow
                        label="Clip"
                        value={`${fmt(camera?.znear)} - ${fmt(camera?.zfar, 1)}`}
                      />
                      <DetailRow label="Speed" value={fmt(camera?.speed, 2)} />
                      <DetailRow
                        label="Sensitivity"
                        value={fmt(camera?.sensitivity, 2)}
                      />
                    </div>
                  </CollapsibleContent>
                </div>
              </Collapsible>
            </CardContent>
          </CollapsibleContent>
        ) : (
          <CardContent className="grid pt-2">
            <div className="grid grid-cols-2">
              <Skeleton className="h-3 w-3" />
              <Skeleton className="h-3 w-25 mr-25" />
            </div>
            <div className="grid grid-cols-2 mt-5">
              <Skeleton className="h-2.5 w-12" />
              <Skeleton className="h-3 w-32" />
            </div>
            <div className="grid gap-2">
              <div className="flex items-center justify-between">
                <Skeleton className="h-3 w-8 mt-5" />
              </div>
              <div className="grid grid-cols-3 gap-2 mt-5">
                <Skeleton className="h-5 w-full rounded-md" />
                <Skeleton className="h-5 w-full rounded-md" />
                <Skeleton className="h-5 w-full rounded-md" />
              </div>
              <div className="flex justify-end gap-1.5">
                <Skeleton className="h-5 w-12 rounded-md" />
                <Skeleton className="h-5 w-12 rounded-md" />
              </div>
            </div>
            <div className="rounded-md border border-border/70 bg-muted/20 p-2.5 mt-5">
              <Skeleton className="h-3 w-12" />
            </div>
          </CardContent>
        )}
      </Card>
    </Collapsible>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-0.5">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-mono tabular-nums">{value}</span>
    </div>
  );
}

export function CameraSkeleton() {
  return (
    <Card
      size="sm"
      className="w-[19rem] max-w-[calc(100vw-2rem)] overflow-hidden ring-0 shadow-[var(--shadow-left)]"
    >
      <CardHeader className="grid-cols-[1fr_auto] items-center pb-0">
        <Skeleton className="h-4 w-16" />
        <Skeleton className="h-5 w-5 rounded-sm" />
      </CardHeader>
      <CardContent className="grid gap-3 pt-2">
        <div className="grid gap-1">
          <Skeleton className="h-2.5 w-10" />
          <Skeleton className="h-3 w-32" />
        </div>
        <div className="grid gap-1">
          <Skeleton className="h-2.5 w-12" />
          <Skeleton className="h-5 w-12 rounded-full" />
        </div>
        <div className="grid gap-2">
          <div className="flex items-center justify-between">
            <Skeleton className="h-3 w-8" />
            <Skeleton className="h-3 w-10" />
          </div>
          <div className="grid grid-cols-3 gap-2">
            <Skeleton className="h-5 w-full rounded-md" />
            <Skeleton className="h-5 w-full rounded-md" />
            <Skeleton className="h-5 w-full rounded-md" />
          </div>
          <div className="flex justify-end gap-1.5">
            <Skeleton className="h-5 w-12 rounded-md" />
            <Skeleton className="h-5 w-12 rounded-md" />
          </div>
        </div>
        <div className="rounded-md border border-border/70 bg-muted/20 p-2.5">
          <Skeleton className="h-3 w-12" />
        </div>
      </CardContent>
    </Card>
  );
}
