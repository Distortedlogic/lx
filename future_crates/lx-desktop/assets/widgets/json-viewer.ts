import type { Widget } from "./registry";
import { registerWidget } from "./registry";

function renderNode(key: string | null, value: unknown, parent: HTMLElement) {
  const row = document.createElement("div");

  if (value !== null && typeof value === "object") {
    const isArray = Array.isArray(value);
    const entries = isArray ? (value as unknown[]) : Object.entries(value as Record<string, unknown>);
    const count = entries.length;
    const label = isArray ? `Array [${count}]` : `Object {${count}}`;

    const toggle = document.createElement("span");
    toggle.textContent = "\u25BC ";
    toggle.style.cursor = "pointer";
    toggle.style.userSelect = "none";

    const keySpan = document.createElement("span");
    if (key !== null) {
      keySpan.textContent = key + ": ";
      keySpan.style.color = "#3b82f6";
    }

    const typeSpan = document.createElement("span");
    typeSpan.textContent = label;
    typeSpan.style.color = "#888";

    row.appendChild(toggle);
    row.appendChild(keySpan);
    row.appendChild(typeSpan);

    const children = document.createElement("div");
    children.style.marginLeft = "16px";

    if (isArray) {
      (value as unknown[]).forEach((item, i) => {
        renderNode(String(i), item, children);
      });
    } else {
      for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
        renderNode(k, v, children);
      }
    }

    toggle.addEventListener("click", () => {
      const collapsed = children.style.display === "none";
      children.style.display = collapsed ? "block" : "none";
      toggle.textContent = collapsed ? "\u25BC " : "\u25B6 ";
    });

    row.appendChild(children);
  } else {
    const keySpan = document.createElement("span");
    if (key !== null) {
      keySpan.textContent = key + ": ";
      keySpan.style.color = "#3b82f6";
    }
    row.appendChild(keySpan);

    const valSpan = document.createElement("span");
    if (typeof value === "string") {
      valSpan.textContent = `"${value}"`;
      valSpan.style.color = "#4ADE80";
    } else if (typeof value === "number") {
      valSpan.textContent = String(value);
      valSpan.style.color = "#F59E0B";
    } else if (typeof value === "boolean") {
      valSpan.textContent = String(value);
      valSpan.style.color = "#A78BFA";
    } else {
      valSpan.textContent = "null";
      valSpan.style.color = "#888";
    }
    row.appendChild(valSpan);
  }

  parent.appendChild(row);
}

const jsonViewerWidget: Widget = {
  mount(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const container = document.createElement("div");
    container.style.overflowY = "auto";
    container.style.height = "100%";
    container.style.padding = "16px";
    container.style.fontFamily = "monospace";
    container.style.fontSize = "13px";
    container.style.background = "#0a0a0a";
    container.style.color = "#e0e0e0";
    container.className = "json-viewer-container";

    el.appendChild(container);
  },

  update(elementId: string, data: unknown) {
    const el = document.getElementById(elementId);
    if (!el) return;
    const container = el.querySelector(".json-viewer-container");
    if (!container) return;
    container.innerHTML = "";
    renderNode(null, data, container as HTMLElement);
  },

  dispose(elementId: string) {
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("json-viewer", jsonViewerWidget);
