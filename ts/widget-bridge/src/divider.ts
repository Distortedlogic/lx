import type { Dioxus } from "./types";

export function startDividerDrag(
  containerId: string,
  direction: "horizontal" | "vertical",
  parentStartPct: number,
  parentSizePct: number,
  dx: Dioxus
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  const containerRect = container.getBoundingClientRect();
  const isHorizontal = direction === "horizontal";

  const axisPos = isHorizontal ? containerRect.x : containerRect.y;
  const axisDim = isHorizontal ? containerRect.width : containerRect.height;
  const parentStart = axisPos + (axisDim * parentStartPct) / 100;
  const parentSize = (axisDim * parentSizePct) / 100;

  function onMove(e: MouseEvent): void {
    const pos = (isHorizontal ? e.clientX : e.clientY) - parentStart;
    const ratio = Math.max(0.1, Math.min(0.9, pos / parentSize));
    dx.send({ type: "ratio", value: ratio });
  }

  function onUp(): void {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    dx.send({ type: "done" });
  }

  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
  document.body.style.cursor = isHorizontal ? "col-resize" : "row-resize";
  document.body.style.userSelect = "none";
}

export async function runDividerDrag(dx: Dioxus): Promise<void> {
  const args = (await dx.recv()) as {
    containerId: string;
    direction: "horizontal" | "vertical";
    parentStart: number;
    parentSize: number;
  };
  startDividerDrag(
    args.containerId,
    args.direction,
    args.parentStart,
    args.parentSize,
    dx
  );
  await dx.recv();
}
