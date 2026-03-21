import type { Dioxus } from "../types";
import type { Widget } from "./registry";
import { registerWidget } from "./registry";

const browserWidget: Widget = {
  mount(elementId: string, config: unknown, dx: Dioxus) {
    const cfg = config as { url?: string; mode?: string; viewport?: { width: number; height: number } };

    if (cfg.mode === "cdp") {
      mountCdp(elementId, cfg, dx);
    } else {
      mountIframe(elementId, cfg, dx);
    }
  },

  update(elementId: string, data: unknown) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const canvas = el.querySelector("canvas");
    if (canvas) {
      const b64 = data as string;
      const img = new Image();
      img.onload = () => {
        const ctx = canvas.getContext("2d");
        if (ctx) {
          canvas.width = img.width;
          canvas.height = img.height;
          ctx.drawImage(img, 0, 0);
        }
      };
      img.src = "data:image/jpeg;base64," + b64;
      return;
    }

    const url = data as string;
    const iframe = el.querySelector("iframe");
    const input = el.querySelector("input");
    if (iframe) iframe.src = url;
    if (input) input.value = url;
  },

  dispose(elementId: string) {
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

function mountIframe(elementId: string, cfg: { url?: string }, dx: Dioxus) {
  const container = document.createElement("div");
  container.style.display = "flex";
  container.style.flexDirection = "column";
  container.style.height = "100%";

  const toolbar = document.createElement("div");
  toolbar.style.height = "36px";
  toolbar.style.display = "flex";
  toolbar.style.alignItems = "center";
  toolbar.style.gap = "4px";
  toolbar.style.padding = "0 8px";
  toolbar.style.background = "#1a1a2e";
  toolbar.style.borderBottom = "1px solid #333";

  const makeBtn = (label: string) => {
    const btn = document.createElement("button");
    btn.innerHTML = label;
    btn.style.background = "none";
    btn.style.border = "none";
    btn.style.color = "#e0e0e0";
    btn.style.cursor = "pointer";
    btn.style.fontSize = "16px";
    btn.style.padding = "2px 6px";
    return btn;
  };

  const backBtn = makeBtn("\u2190");
  const fwdBtn = makeBtn("\u2192");
  const refreshBtn = makeBtn("\u21BB");

  const input = document.createElement("input");
  input.type = "text";
  input.style.flex = "1";
  input.style.background = "#0a0a0a";
  input.style.border = "1px solid #444";
  input.style.color = "#e0e0e0";
  input.style.fontSize = "13px";
  input.style.padding = "4px 8px";
  input.style.borderRadius = "4px";

  toolbar.appendChild(backBtn);
  toolbar.appendChild(fwdBtn);
  toolbar.appendChild(refreshBtn);
  toolbar.appendChild(input);

  const iframe = document.createElement("iframe");
  iframe.style.flex = "1";
  iframe.style.width = "100%";
  iframe.style.border = "none";
  iframe.style.background = "white";

  container.appendChild(toolbar);
  container.appendChild(iframe);

  const el = document.getElementById(elementId);
  if (el) el.appendChild(container);

  const initialUrl = cfg?.url ?? "";
  iframe.src = initialUrl;
  input.value = initialUrl;

  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      iframe.src = input.value;
      dx.send({ type: "navigate", url: input.value });
    }
  });

  backBtn.addEventListener("click", () => {
    try { iframe.contentWindow?.history.back(); } catch {}
  });

  fwdBtn.addEventListener("click", () => {
    try { iframe.contentWindow?.history.forward(); } catch {}
  });

  refreshBtn.addEventListener("click", () => {
    try { iframe.contentWindow?.location.reload(); } catch {}
  });
}

function mountCdp(elementId: string, cfg: { url?: string; viewport?: { width: number; height: number } }, dx: Dioxus) {
  const container = document.createElement("div");
  container.style.display = "flex";
  container.style.flexDirection = "column";
  container.style.height = "100%";

  const toolbar = document.createElement("div");
  toolbar.style.height = "36px";
  toolbar.style.display = "flex";
  toolbar.style.alignItems = "center";
  toolbar.style.gap = "4px";
  toolbar.style.padding = "0 8px";
  toolbar.style.background = "#1a1a2e";
  toolbar.style.borderBottom = "1px solid #333";

  const makeBtn = (label: string) => {
    const btn = document.createElement("button");
    btn.innerHTML = label;
    btn.style.background = "none";
    btn.style.border = "none";
    btn.style.color = "#e0e0e0";
    btn.style.cursor = "pointer";
    btn.style.fontSize = "16px";
    btn.style.padding = "2px 6px";
    return btn;
  };

  const backBtn = makeBtn("\u2190");
  const fwdBtn = makeBtn("\u2192");
  const refreshBtn = makeBtn("\u21BB");

  const input = document.createElement("input");
  input.type = "text";
  input.style.flex = "1";
  input.style.background = "#0a0a0a";
  input.style.border = "1px solid #444";
  input.style.color = "#e0e0e0";
  input.style.fontSize = "13px";
  input.style.padding = "4px 8px";
  input.style.borderRadius = "4px";
  input.value = cfg?.url ?? "";

  toolbar.appendChild(backBtn);
  toolbar.appendChild(fwdBtn);
  toolbar.appendChild(refreshBtn);
  toolbar.appendChild(input);

  const wrapper = document.createElement("div");
  wrapper.style.flex = "1";
  wrapper.style.position = "relative";
  wrapper.style.overflow = "hidden";

  const canvas = document.createElement("canvas");
  canvas.style.width = "100%";
  canvas.style.height = "100%";
  canvas.style.display = "block";
  canvas.style.cursor = "crosshair";

  const textarea = document.createElement("textarea");
  textarea.style.position = "absolute";
  textarea.style.top = "0";
  textarea.style.left = "0";
  textarea.style.width = "100%";
  textarea.style.height = "100%";
  textarea.style.opacity = "0";
  textarea.style.cursor = "crosshair";
  textarea.style.resize = "none";

  wrapper.appendChild(canvas);
  wrapper.appendChild(textarea);
  container.appendChild(toolbar);
  container.appendChild(wrapper);

  const el = document.getElementById(elementId);
  if (el) el.appendChild(container);

  canvas.addEventListener("click", (e) => {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const x = (e.clientX - rect.left) * scaleX;
    const y = (e.clientY - rect.top) * scaleY;
    dx.send({ type: "click", x, y });
  });

  textarea.addEventListener("input", () => {
    const text = textarea.value;
    if (text) {
      dx.send({ type: "type", text });
      textarea.value = "";
    }
  });

  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      dx.send({ type: "navigate", url: input.value });
    }
  });

  backBtn.addEventListener("click", () => dx.send({ type: "back" }));
  fwdBtn.addEventListener("click", () => dx.send({ type: "forward" }));
  refreshBtn.addEventListener("click", () => dx.send({ type: "refresh" }));
}

registerWidget("browser", browserWidget);
