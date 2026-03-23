import { marked } from "marked";
import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";

const markdownWidget: Widget = {
  mount(elementId: string) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const style = document.createElement("style");
    style.textContent = `
      #${elementId} .md-container h1 { font-size: 2em; font-weight: 700; margin: 0.67em 0; }
      #${elementId} .md-container h2 { font-size: 1.5em; font-weight: 700; margin: 0.83em 0; }
      #${elementId} .md-container h3 { font-size: 1.17em; font-weight: 700; margin: 1em 0; }
      #${elementId} .md-container h4 { font-size: 1em; font-weight: 700; margin: 1.33em 0; }
      #${elementId} .md-container h5 { font-size: 0.83em; font-weight: 700; margin: 1.67em 0; }
      #${elementId} .md-container h6 { font-size: 0.67em; font-weight: 700; margin: 2.33em 0; }
      #${elementId} .md-container code { background: #1a1a2e; font-family: monospace; padding: 2px 6px; }
      #${elementId} .md-container pre { background: #1a1a2e; padding: 12px; overflow-x: auto; }
      #${elementId} .md-container pre code { padding: 0; background: none; }
      #${elementId} .md-container a { color: #3b82f6; }
      #${elementId} .md-container blockquote { border-left: 3px solid #666; padding-left: 12px; color: #999; }
    `;

    const container = document.createElement("div");
    container.className = "md-container";
    container.style.overflowY = "auto";
    container.style.height = "100%";
    container.style.padding = "24px";
    container.style.color = "#e0e0e0";
    container.style.background = "#0a0a0a";

    el.appendChild(style);
    el.appendChild(container);
    container.innerHTML = '<p style="color: #757575;">Markdown viewer — no content loaded</p>';
  },

  update(elementId: string, data: unknown) {
    const el = document.getElementById(elementId);
    if (!el) return;
    const container = el.querySelector(".md-container");
    if (!container) return;
    container.innerHTML = marked.parse(data as string) as string;
  },

  dispose(elementId: string) {
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("markdown", markdownWidget);
