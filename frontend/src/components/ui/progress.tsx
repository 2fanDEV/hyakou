import { cn } from "#/lib/utils";

function Progress({
	className,
	indicatorClassName,
	value,
	...props
}: React.ComponentProps<"div"> & {
	indicatorClassName?: string;
	value?: number | null;
}) {
	const progressValue =
		value == null ? null : Math.max(0, Math.min(100, Math.round(value)));

	return (
		<div
			data-slot="progress"
			data-indeterminate={progressValue == null || undefined}
			className={cn(
				"bg-primary/10 relative h-1.5 w-full overflow-hidden rounded-full",
				className,
			)}
			{...props}
		>
			<div
				data-slot="progress-indicator"
				className={cn(
					"bg-primary h-full w-full origin-left transition-transform duration-150 ease-out",
					progressValue == null && "animate-pulse",
					indicatorClassName,
				)}
				style={
					progressValue == null
						? undefined
						: { transform: `translateX(${progressValue - 100}%)` }
				}
			/>
		</div>
	);
}

export { Progress };
