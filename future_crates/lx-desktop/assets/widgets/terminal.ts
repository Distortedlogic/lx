import type { Dioxus } from "../types";
import type { Widget } from "./registry";
import { mountTerminal, writeTerminal, fitTerminal, disposeTerminal } from "../terminal";
import { registerWidget } from "./registry";

const terminalWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    mountTerminal(elementId, dx);
  },
  update(elementId: string, data: unknown) {
    writeTerminal(elementId, data as string);
  },
  resize(elementId: string) {
    fitTerminal(elementId);
  },
  dispose(elementId: string) {
    disposeTerminal(elementId);
  },
};

registerWidget("terminal", terminalWidget);
