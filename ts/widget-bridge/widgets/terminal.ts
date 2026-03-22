import type { Dioxus } from "../src/types";
import type { Widget } from "../src/registry";
import { mountTerminal, writeTerminal, fitTerminal, disposeTerminal } from "../src/terminal";
import { registerWidget } from "../src/registry";

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
