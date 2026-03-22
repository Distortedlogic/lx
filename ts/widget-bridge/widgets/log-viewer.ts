import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";

interface LogLine {
  level: string;
  message: string;
  ts?: string;
}

interface LogViewerState {
  container: HTMLDivElement;
  userScrolled: boolean;
}

const states = new Map<string, LogViewerState>();

const levelColors: Record<string, string> = {
  info: "#e0e0e0",
  warn: "#F59E0B",
  error: "#EF4444",
  debug: "#888",
};

function appendLine(state: LogViewerState, line: LogLine) {
  const div = document.createElement("div");
  div.style.color = levelColors[line.level] ?? "#e0e0e0";

  if (line.ts) {
    const tsSpan = document.createElement("span");
    tsSpan.style.color = "gray";
    tsSpan.textContent = line.ts + " ";
    div.appendChild(tsSpan);
  }

  const msgSpan = document.createElement("span");
  msgSpan.textContent = line.message;
  div.appendChild(msgSpan);

  state.container.appendChild(div);

  if (!state.userScrolled) {
    state.container.scrollTop = state.container.scrollHeight;
  }
}

const logViewerWidget: Widget = {
  mount(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const container = document.createElement("div");
    container.style.overflowY = "auto";
    container.style.height = "100%";
    container.style.background = "#0a0a0a";
    container.style.fontFamily = "monospace";
    container.style.fontSize = "13px";
    container.style.padding = "8px";
    container.style.lineHeight = "1.4";

    el.appendChild(container);

    const state: LogViewerState = { container, userScrolled: false };

    container.addEventListener("scroll", () => {
      const atBottom =
        container.scrollHeight - container.scrollTop - container.clientHeight < 30;
      state.userScrolled = !atBottom;
    });

    states.set(elementId, state);
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;

    if (Array.isArray(data)) {
      for (const line of data as LogLine[]) {
        appendLine(state, line);
      }
    } else {
      appendLine(state, data as LogLine);
    }
  },

  dispose(elementId: string) {
    states.delete(elementId);
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("log-viewer", logViewerWidget);
