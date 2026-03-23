import type { Dioxus } from "../src/types";
import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";

const editors = new Map<string, { container: HTMLDivElement; content: string }>();

const editorWidget: Widget = {
  mount(elementId: string, config: unknown, dx: Dioxus) {
    const cfg = config as { content?: string; language?: string; filePath?: string };
    const container = document.createElement("div");
    container.style.width = "100%";
    container.style.height = "100%";
    container.style.overflow = "auto";
    container.style.background = "#1a1a1a";
    container.style.color = "#e0e0e0";
    container.style.fontFamily = "monospace";
    container.style.fontSize = "14px";
    container.style.padding = "16px";
    container.style.whiteSpace = "pre-wrap";
    container.style.wordWrap = "break-word";

    const el = document.getElementById(elementId);
    if (el) el.appendChild(container);

    const content = cfg.content ?? "";
    container.textContent = content;
    container.contentEditable = "true";
    container.spellcheck = false;
    container.classList.add("border-l-2", "border-[var(--outline-variant)]");

    const placeholderText = "Empty — start typing";

    function updatePlaceholder() {
      if (!container.textContent) {
        container.style.color = "#757575";
        container.textContent = placeholderText;
      }
    }

    container.addEventListener("focus", () => {
      if (container.textContent === placeholderText) {
        container.textContent = "";
        container.style.color = "#e0e0e0";
      }
    });

    container.addEventListener("blur", () => {
      updatePlaceholder();
    });

    updatePlaceholder();

    container.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        dx.send({ type: "save", content: container.textContent ?? "" });
      }
    });

    editors.set(elementId, { container, content });
  },

  update(elementId: string, data: unknown) {
    const d = data as { content?: string };
    const state = editors.get(elementId);
    if (state && d.content !== undefined) {
      state.container.textContent = d.content;
      state.content = d.content;
    }
  },

  resize(_elementId: string) {},

  dispose(elementId: string) {
    editors.delete(elementId);
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("editor", editorWidget);
