import { SnapModifier } from "@dnd-kit/abstract/modifiers";
import { RestrictToWindow } from "@dnd-kit/dom/modifiers";
import {
	DragDropProvider,
	useDragDropMonitor,
	useDraggable,
} from "@dnd-kit/react";
import { CaretDownIcon, CaretUpIcon } from "@phosphor-icons/react";
import { type ReactNode, useState } from "react";
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from "#/components/ui/collapsible";
import { cn } from "#/lib/utils";

type PanelPosition = {
	x: number;
	y: number;
};

type Props = {
	title: string;
	children: ReactNode;
	headerActions?: ReactNode;
	defaultWidth?: number;
	defaultHeight?: number;
	className?: string;
};

const DEFAULT_WIDTH = 240;
const DEFAULT_HEIGHT = 320;
const GRID_SIZE = 2;
const PANEL_MARGIN = 2;

function getDefaultPosition(width: number): PanelPosition {
	const winW = typeof window !== "undefined" ? window.innerWidth : 1280;
	const rawX = winW - width - PANEL_MARGIN;
	return {
		x: Math.round(rawX / GRID_SIZE) * GRID_SIZE,
		y: PANEL_MARGIN,
	};
}

function DraggablePanelBody({
	title,
	children,
	headerActions,
	defaultWidth,
	defaultHeight,
	className,
}: Required<Pick<Props, "title" | "children">> &
	Pick<Props, "headerActions" | "className"> & {
		defaultWidth: number;
		defaultHeight: number;
	}) {
	const [position, setPosition] = useState<PanelPosition>(() =>
		getDefaultPosition(defaultWidth),
	);
	const [dragOffset, setDragOffset] = useState<PanelPosition>({ x: 0, y: 0 });
	const [open, setOpen] = useState(true);
	const panelId = `panel:${title}`;
	const { ref, handleRef, isDragging } = useDraggable({
		id: panelId,
		modifiers: [RestrictToWindow, SnapModifier.configure({ size: GRID_SIZE })],
	});

	useDragDropMonitor({
		onDragMove(event) {
			if (event.operation.source?.id !== panelId) return;
			setDragOffset(event.operation.transform);
		},
		onDragEnd(event) {
			if (event.operation.source?.id !== panelId) return;
			if (event.canceled) {
				setDragOffset({ x: 0, y: 0 });
				return;
			}
			setPosition((prev) => ({
				x: prev.x + event.operation.transform.x,
				y: prev.y + event.operation.transform.y,
			}));
			setDragOffset({ x: 0, y: 0 });
		},
		onDragStart(event) {
			if (event.operation.source?.id !== panelId) return;
			setDragOffset({ x: 0, y: 0 });
		},
	});

	return (
		<Collapsible open={open} onOpenChange={setOpen}>
			<div
				ref={ref}
				className={cn("draggable-panel", className, isDragging && "opacity-95")}
				style={{
					position: "fixed",
					left: position.x,
					top: position.y,
					translate: `${dragOffset.x}px ${dragOffset.y}px`,
					width: defaultWidth,
					zIndex: 20,
				}}
			>
				<div className="draggable-panel-header">
					<div ref={handleRef} className="draggable-panel-handle">
						<span className="draggable-panel-title">{title}</span>
					</div>
					<div className="flex items-center gap-1">
						{headerActions}
						<CollapsibleTrigger asChild>
							<button
								type="button"
								className="draggable-panel-toggle"
								aria-label={open ? "Minimize panel" : "Expand panel"}
							>
								{open ? <CaretUpIcon size={10} /> : <CaretDownIcon size={10} />}
							</button>
						</CollapsibleTrigger>
					</div>
				</div>

				<CollapsibleContent className="animate-slide-down overflow-hidden data-[state=closed]:animate-slide-up">
					<div
						className="draggable-panel-body"
						style={{ maxHeight: defaultHeight, overflowY: "auto" }}
					>
						{children}
					</div>
				</CollapsibleContent>
			</div>
		</Collapsible>
	);
}

export function DraggablePanel({
	title,
	children,
	headerActions,
	defaultWidth = DEFAULT_WIDTH,
	defaultHeight = DEFAULT_HEIGHT,
	className,
}: Props) {
	return (
		<DragDropProvider>
			<DraggablePanelBody
				title={title}
				headerActions={headerActions}
				defaultWidth={defaultWidth}
				defaultHeight={defaultHeight}
				className={className}
			>
				{children}
			</DraggablePanelBody>
		</DragDropProvider>
	);
}
