import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";

const instances = new Map<string, { container: HTMLElement; toolbar: HTMLElement }>();

function renderNode(key: string | null, value: unknown, parent: HTMLElement, path: string) {
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
    toggle.dataset.role = "toggle";

    const keySpan = document.createElement("span");
    if (key !== null) {
      keySpan.textContent = key + ": ";
      keySpan.style.color = "#3b82f6";
      keySpan.style.cursor = "pointer";
      keySpan.addEventListener("click", () => {
        navigator.clipboard.writeText(path).catch(() => {});
        keySpan.style.background = "#484848";
        setTimeout(() => { keySpan.style.background = ""; }, 300);
      });
    }

    const typeSpan = document.createElement("span");
    typeSpan.textContent = label;
    typeSpan.style.color = "#888";

    row.appendChild(toggle);
    row.appendChild(keySpan);
    row.appendChild(typeSpan);

    const children = document.createElement("div");
    children.style.marginLeft = "16px";
    children.dataset.role = "children";

    if (isArray) {
      (value as unknown[]).forEach((item, i) => {
        const childPath = path ? `${path}[${i}]` : `[${i}]`;
        renderNode(String(i), item, children, childPath);
      });
    } else {
      for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
        const childPath = path ? `${path}.${k}` : k;
        renderNode(k, v, children, childPath);
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
      keySpan.style.cursor = "pointer";
      keySpan.addEventListener("click", () => {
        navigator.clipboard.writeText(path).catch(() => {});
        keySpan.style.background = "#484848";
        setTimeout(() => { keySpan.style.background = ""; }, 300);
      });
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

function createButton(text: string, onClick: () => void): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.textContent = text;
  btn.style.background = "#1a1a1a";
  btn.style.color = "#e0e0e0";
  btn.style.border = "1px solid #484848";
  btn.style.borderRadius = "3px";
  btn.style.padding = "2px 8px";
  btn.style.cursor = "pointer";
  btn.style.fontSize = "11px";
  btn.addEventListener("click", onClick);
  return btn;
}

const jsonViewerWidget: Widget = {
  mount(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) return;

    el.style.display = "flex";
    el.style.flexDirection = "column";
    el.style.height = "100%";

    const toolbar = document.createElement("div");
    toolbar.style.display = "flex";
    toolbar.style.alignItems = "center";
    toolbar.style.gap = "4px";
    toolbar.style.padding = "4px 8px";
    toolbar.style.background = "#131313";
    toolbar.style.borderBottom = "1px solid #484848";
    toolbar.style.fontSize = "11px";

    const container = document.createElement("div");
    container.style.flex = "1";
    container.style.overflowY = "auto";
    container.style.padding = "16px";
    container.style.fontFamily = "monospace";
    container.style.fontSize = "13px";
    container.style.background = "#0a0a0a";
    container.style.color = "#e0e0e0";
    container.className = "json-viewer-container";

    const searchInput = document.createElement("input");
    searchInput.type = "text";
    searchInput.placeholder = "Search...";
    searchInput.style.background = "#0a0a0a";
    searchInput.style.border = "1px solid #484848";
    searchInput.style.color = "#e0e0e0";
    searchInput.style.fontSize = "13px";
    searchInput.style.flex = "1";
    searchInput.style.padding = "2px 6px";
    searchInput.style.borderRadius = "3px";
    searchInput.style.outline = "none";

    searchInput.addEventListener("input", () => {
      const query = searchInput.value.toLowerCase();
      const rows = container.querySelectorAll("div");

      if (!query) {
        rows.forEach((row) => { (row as HTMLElement).style.display = ""; });
        return;
      }

      rows.forEach((row) => { (row as HTMLElement).style.display = "none"; });

      rows.forEach((row) => {
        const spans = row.querySelectorAll(":scope > span");
        let matches = false;
        spans.forEach((span) => {
          if (span.textContent && span.textContent.toLowerCase().includes(query)) {
            matches = true;
          }
        });
        if (matches) {
          (row as HTMLElement).style.display = "";
          let ancestor = row.parentElement;
          while (ancestor && ancestor !== container) {
            (ancestor as HTMLElement).style.display = "";
            ancestor = ancestor.parentElement;
          }
        }
      });
    });

    const expandAllBtn = createButton("Expand All", () => {
      container.querySelectorAll('[data-role="children"]').forEach((child) => {
        (child as HTMLElement).style.display = "block";
      });
      container.querySelectorAll('[data-role="toggle"]').forEach((tog) => {
        tog.textContent = "\u25BC ";
      });
    });

    const collapseAllBtn = createButton("Collapse All", () => {
      container.querySelectorAll('[data-role="children"]').forEach((child) => {
        (child as HTMLElement).style.display = "none";
      });
      container.querySelectorAll('[data-role="toggle"]').forEach((tog) => {
        tog.textContent = "\u25B6 ";
      });
    });

    toolbar.appendChild(searchInput);
    toolbar.appendChild(expandAllBtn);
    toolbar.appendChild(collapseAllBtn);

    el.appendChild(toolbar);
    el.appendChild(container);

    instances.set(elementId, { container, toolbar });

    const placeholder = document.createElement("div");
    placeholder.textContent = "JSON viewer \u2014 no data loaded";
    placeholder.style.color = "#757575";
    container.appendChild(placeholder);
  },

  update(elementId: string, data: unknown) {
    const el = document.getElementById(elementId);
    if (!el) return;
    const container = el.querySelector(".json-viewer-container");
    if (!container) return;
    container.innerHTML = "";
    renderNode(null, data, container as HTMLElement, "");
  },

  dispose(elementId: string) {
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
    instances.delete(elementId);
  },
};

registerWidget("json-viewer", jsonViewerWidget);
