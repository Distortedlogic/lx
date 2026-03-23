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
  hiddenLevels: Set<string>;
  lineCountSpan: HTMLSpanElement;
  placeholder: HTMLElement | null;
}

const states = new Map<string, LogViewerState>();

const levelColors: Record<string, string> = {
  info: "#4fc3f7",
  warn: "#ffb74d",
  error: "#ef5350",
  debug: "#81c784",
};

function updateLineCount(state: LogViewerState) {
  const total = state.container.querySelectorAll("[data-level]").length;
  const hidden = state.hiddenLevels.size > 0
    ? Array.from(state.container.querySelectorAll("[data-level]")).filter(
        (el) => state.hiddenLevels.has((el as HTMLElement).dataset.level ?? "")
      ).length
    : 0;
  if (hidden > 0) {
    state.lineCountSpan.textContent = `${total - hidden} / ${total} lines`;
  } else {
    state.lineCountSpan.textContent = `${total} lines`;
  }
}

function appendLine(state: LogViewerState, line: LogLine) {
  const div = document.createElement("div");
  div.style.color = levelColors[line.level] ?? "#e0e0e0";
  div.dataset.level = line.level;

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

function createPlaceholder(): HTMLParagraphElement {
  const placeholder = document.createElement("p");
  placeholder.style.color = "#757575";
  placeholder.textContent = "Log viewer — awaiting log entries";
  return placeholder;
}

const CONTAINER_CLASS = "lx-log-viewer";

const FILTER_CSS = `
.${CONTAINER_CLASS}.hide-info [data-level="info"] { display: none }
.${CONTAINER_CLASS}.hide-warn [data-level="warn"] { display: none }
.${CONTAINER_CLASS}.hide-error [data-level="error"] { display: none }
.${CONTAINER_CLASS}.hide-debug [data-level="debug"] { display: none }
`;

const LEVELS = ["info", "warn", "error", "debug"] as const;

function createToolbar(state: LogViewerState): HTMLDivElement {
  const toolbar = document.createElement("div");
  toolbar.style.display = "flex";
  toolbar.style.alignItems = "center";
  toolbar.style.gap = "4px";
  toolbar.style.padding = "4px 8px";
  toolbar.style.background = "#131313";
  toolbar.style.borderBottom = "1px solid #484848";
  toolbar.style.fontSize = "11px";

  for (const level of LEVELS) {
    const btn = document.createElement("button");
    btn.textContent = level;
    btn.style.background = "none";
    btn.style.border = `1px solid ${levelColors[level]}`;
    btn.style.color = levelColors[level];
    btn.style.borderRadius = "3px";
    btn.style.padding = "1px 6px";
    btn.style.cursor = "pointer";
    btn.style.opacity = "1";
    btn.style.fontSize = "11px";
    btn.addEventListener("click", () => {
      if (state.hiddenLevels.has(level)) {
        state.hiddenLevels.delete(level);
        btn.style.opacity = "1";
        state.container.classList.remove(`hide-${level}`);
      } else {
        state.hiddenLevels.add(level);
        btn.style.opacity = "0.3";
        state.container.classList.add(`hide-${level}`);
      }
      updateLineCount(state);
    });
    toolbar.appendChild(btn);
  }

  const clearBtn = document.createElement("button");
  clearBtn.textContent = "Clear";
  clearBtn.style.background = "none";
  clearBtn.style.border = "1px solid #757575";
  clearBtn.style.color = "#757575";
  clearBtn.style.borderRadius = "3px";
  clearBtn.style.padding = "1px 6px";
  clearBtn.style.cursor = "pointer";
  clearBtn.style.marginLeft = "auto";
  clearBtn.style.fontSize = "11px";
  clearBtn.addEventListener("click", () => {
    state.container.innerHTML = "";
    const placeholder = createPlaceholder();
    state.container.appendChild(placeholder);
    state.placeholder = placeholder;
    updateLineCount(state);
  });
  toolbar.appendChild(clearBtn);

  const lineCountSpan = document.createElement("span");
  lineCountSpan.style.color = "#757575";
  lineCountSpan.style.marginLeft = "8px";
  lineCountSpan.textContent = "0 lines";
  state.lineCountSpan = lineCountSpan;
  toolbar.appendChild(lineCountSpan);

  return toolbar;
}

const logViewerWidget: Widget = {
  mount(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) return;

    el.style.display = "flex";
    el.style.flexDirection = "column";
    el.style.height = "100%";

    const style = document.createElement("style");
    style.textContent = FILTER_CSS;
    el.appendChild(style);

    const container = document.createElement("div");
    container.className = CONTAINER_CLASS;
    container.style.flex = "1";
    container.style.overflowY = "auto";
    container.style.background = "#0a0a0a";
    container.style.fontFamily = "monospace";
    container.style.fontSize = "13px";
    container.style.padding = "8px";
    container.style.lineHeight = "1.4";

    const placeholder = createPlaceholder();

    const state: LogViewerState = {
      container,
      userScrolled: false,
      hiddenLevels: new Set(),
      lineCountSpan: document.createElement("span"),
      placeholder,
    };

    const toolbar = createToolbar(state);
    el.appendChild(toolbar);
    el.appendChild(container);

    container.appendChild(placeholder);

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

    if (state.placeholder) {
      state.placeholder.remove();
      state.placeholder = null;
    }

    if (Array.isArray(data)) {
      for (const line of data as LogLine[]) {
        appendLine(state, line);
      }
    } else {
      appendLine(state, data as LogLine);
    }

    updateLineCount(state);
  },

  dispose(elementId: string) {
    states.delete(elementId);
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("log-viewer", logViewerWidget);
