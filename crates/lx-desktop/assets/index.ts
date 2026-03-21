export { mountTerminal, writeTerminal, disposeTerminal, fitTerminal } from "./terminal";
export { startDividerDrag, runDividerDrag } from "./divider";
export { runWidgetBridge, registerWidget } from "./widgets/registry";
export type { Widget } from "./widgets/registry";

import "./widgets/terminal";
import "./widgets/browser";
import "./widgets/editor";
import "./widgets/agent";
import "./widgets/log-viewer";
import "./widgets/markdown";
import "./widgets/json-viewer";
import "./widgets/flow-graph";

import type * as Desktop from "./index";
import * as self from "./index";

declare global {
  interface Window {
    LxDesktop: typeof Desktop;
  }
}

window.LxDesktop = self;
