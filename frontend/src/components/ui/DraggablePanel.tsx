import { CaretDownIcon, CaretUpIcon } from "@phosphor-icons/react";
import {
	type PointerEvent,
	type ReactNode,
	useCallback,
	useRef,
	useState,
} from "react";
import { cn } from "#/lib/utils";

type PanelGeometry = {
	x: number;
	y: number;
	width: number;
	height: number;
};

type DragState = {
	startX: number;
	startY: number;
	startPanelX: number;
	startPanelY: number;
};

type ResizeDir = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

type ResizeState = {
	startX: number;
	startY: number;
	startW: number;
	startH: number;
	startPanelX: number;
	startPanelY: number;
	dir: ResizeDir;
};

type Props = {
	title: string;
	children: ReactNode;
	headerActions?: ReactNode;
	defaultWidth?: number;
	defaultHeight?: number;
	minWidth?: number;
	minHeight?: number;
	className?: string;
};

const DEFAULT_WIDTH = 240;
const DEFAULT_HEIGHT = 320;
const MIN_WIDTH = 160;
const MIN_HEIGHT = 80;
const PANEL_MARGIN = 16;

function getDefaultGeometry(width: number, height: number): PanelGeometry {
	const winW = typeof window !== "undefined" ? window.innerWidth : 1280;
	return {
		x: winW - width - PANEL_MARGIN,
		y: PANEL_MARGIN,
		width,
		height,
	};
}

const RESIZE_CURSORS: Record<ResizeDir, string> = {
	n: "ns-resize",
	s: "ns-resize",
	e: "ew-resize",
	w: "ew-resize",
	ne: "nesw-resize",
	nw: "nwse-resize",
	se: "nwse-resize",
	sw: "nesw-resize",
};

export function DraggablePanel({
	title,
	children,
	headerActions,
	defaultWidth = DEFAULT_WIDTH,
	defaultHeight = DEFAULT_HEIGHT,
	minWidth = MIN_WIDTH,
	minHeight = MIN_HEIGHT,
	className,
}: Props) {
	const [geo, setGeo] = useState<PanelGeometry>(() =>
		getDefaultGeometry(defaultWidth, defaultHeight),
	);
	const [minimized, setMinimized] = useState(false);

	const dragRef = useRef<DragState | null>(null);
	const resizeRef = useRef<ResizeState | null>(null);

	const onHeaderPointerDown = useCallback(
		(e: PointerEvent<HTMLDivElement>) => {
			e.currentTarget.setPointerCapture(e.pointerId);
			dragRef.current = {
				startX: e.clientX,
				startY: e.clientY,
				startPanelX: geo.x,
				startPanelY: geo.y,
			};
		},
		[geo.x, geo.y],
	);

	const onHeaderPointerMove = useCallback((e: PointerEvent<HTMLDivElement>) => {
		const ds = dragRef.current;
		if (!ds) return;
		const dx = e.clientX - ds.startX;
		const dy = e.clientY - ds.startY;
		setGeo((prev) => ({
			...prev,
			x: ds.startPanelX + dx,
			y: ds.startPanelY + dy,
		}));
	}, []);

	const onHeaderPointerUp = useCallback(() => {
		dragRef.current = null;
	}, []);

	const onResizePointerDown = useCallback(
		(e: PointerEvent<HTMLDivElement>, dir: ResizeDir) => {
			e.preventDefault();
			e.stopPropagation();
			e.currentTarget.setPointerCapture(e.pointerId);
			resizeRef.current = {
				startX: e.clientX,
				startY: e.clientY,
				startW: geo.width,
				startH: geo.height,
				startPanelX: geo.x,
				startPanelY: geo.y,
				dir,
			};
		},
		[geo],
	);

	const onResizePointerMove = useCallback(
		(e: PointerEvent<HTMLDivElement>) => {
			const rs = resizeRef.current;
			if (!rs) return;
			const dx = e.clientX - rs.startX;
			const dy = e.clientY - rs.startY;

			setGeo((prev) => {
				let { x, y, width, height } = prev;

				if (rs.dir.includes("e")) width = Math.max(minWidth, rs.startW + dx);
				if (rs.dir.includes("w")) {
					const newW = Math.max(minWidth, rs.startW - dx);
					x = rs.startPanelX + rs.startW - newW;
					width = newW;
				}
				if (rs.dir.includes("s")) height = Math.max(minHeight, rs.startH + dy);
				if (rs.dir.includes("n")) {
					const newH = Math.max(minHeight, rs.startH - dy);
					y = rs.startPanelY + rs.startH - newH;
					height = newH;
				}

				return { x, y, width, height };
			});
		},
		[minWidth, minHeight],
	);

	const onResizePointerUp = useCallback(() => {
		resizeRef.current = null;
	}, []);

	return (
		<div
			className={cn("draggable-panel", className)}
			style={{
				position: "fixed",
				left: geo.x,
				top: geo.y,
				width: geo.width,
				zIndex: 20,
			}}
		>
			<div
				className="draggable-panel-header"
				onPointerDown={onHeaderPointerDown}
				onPointerMove={onHeaderPointerMove}
				onPointerUp={onHeaderPointerUp}
			>
				<span className="draggable-panel-title">{title}</span>
				<div className="flex items-center gap-1">
					{headerActions}
					<button
						type="button"
						className="draggable-panel-toggle"
						onClick={() => setMinimized((v) => !v)}
						aria-label={minimized ? "Expand panel" : "Minimize panel"}
					>
						{minimized ? (
							<CaretDownIcon size={10} />
						) : (
							<CaretUpIcon size={10} />
						)}
					</button>
				</div>
			</div>

			{!minimized && (
				<div
					className="draggable-panel-body"
					style={{ height: geo.height, overflowY: "auto" }}
				>
					{children}
				</div>
			)}

			{!minimized &&
				(["n", "s", "e", "w", "ne", "nw", "se", "sw"] as ResizeDir[]).map(
					(dir) => (
						<div
							key={dir}
							className="resize-handle"
							data-dir={dir}
							style={{ cursor: RESIZE_CURSORS[dir] }}
							onPointerDown={(e) => onResizePointerDown(e, dir)}
							onPointerMove={onResizePointerMove}
							onPointerUp={onResizePointerUp}
						/>
					),
				)}
		</div>
	);
}
