export { mountTerminal, writeTerminal, disposeTerminal, fitTerminal } from "./terminal";
export { startDividerDrag, runDividerDrag } from "./divider";
export { runWidgetBridge, registerWidget } from "./registry";
export type { Widget } from "./registry";
export type { Dioxus } from "./types";

import "../widgets/terminal";
import "../widgets/browser";
import "../widgets/editor";
import "../widgets/agent";
import "../widgets/log-viewer";
import "../widgets/markdown";
import "../widgets/json-viewer";
import "../widgets/voice";

import * as self from "./index";

declare global {
  interface Window {
    WidgetBridge: typeof self;
  }
}

window.WidgetBridge = self;
