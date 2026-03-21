import type { Dioxus } from "../types";

export interface Widget {
  mount(elementId: string, config: unknown, dx: Dioxus): void;
  update(elementId: string, data: unknown): void;
  resize?(elementId: string): void;
  dispose(elementId: string): void;
}

const widgets = new Map<string, Widget>();

export function registerWidget(name: string, widget: Widget) {
  widgets.set(name, widget);
}

export async function runWidgetBridge(dx: Dioxus): Promise<void> {
  const init = (await dx.recv()) as {
    widget: string;
    elementId: string;
    config: unknown;
  };

  const w = widgets.get(init.widget);
  if (!w) throw new Error(`unknown widget: ${init.widget}`);

  while (!document.getElementById(init.elementId)) {
    await new Promise((r) => setTimeout(r, 10));
  }

  w.mount(init.elementId, init.config, dx);

  try {
    while (true) {
      const msg = (await dx.recv()) as { action: string; data?: unknown };
      switch (msg.action) {
        case "update":
          w.update(init.elementId, msg.data);
          break;
        case "resize":
          w.resize?.(init.elementId);
          break;
        case "dispose":
          w.dispose(init.elementId);
          return;
      }
    }
  } catch {
    w.dispose(init.elementId);
  }
}
