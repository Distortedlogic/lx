import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { ensureXtermCss } from "./inject-css";
import type { Dioxus } from "./types";

interface TerminalInstance {
  term: Terminal;
  fitAddon: FitAddon;
  ro: ResizeObserver;
}

const instances = new Map<string, TerminalInstance>();

export function mountTerminal(elementId: string, dx: Dioxus): void {
  if (instances.has(elementId)) return;
  ensureXtermCss();

  const container = document.getElementById(elementId);
  if (!container) throw new Error(`terminal container not found: ${elementId}`);

  const term = new Terminal({
    cursorBlink: true,
    theme: {
      background: "#0E0E0E",
      foreground: "#DCC1AE",
      cursor: "#FFB87B",
      cursorAccent: "#0E0E0E",
      selectionBackground: "rgba(255,184,123,0.2)",
      selectionForeground: "#E6E1DD",
    },
  });

  const fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(container);
  fitAddon.fit();

  let rafId: number | null = null;
  const ro = new ResizeObserver(() => {
    if (rafId !== null) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(() => {
      rafId = null;
      fitAddon.fit();
      dx.send({ type: "resize", cols: term.cols, rows: term.rows });
    });
  });
  ro.observe(container);

  term.onData((data: string) => {
    dx.send({ type: "input", data });
  });

  instances.set(elementId, { term, fitAddon, ro });
}

export function writeTerminal(elementId: string, b64data: string): void {
  const inst = instances.get(elementId);
  if (!inst) return;
  const binary = atob(b64data);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  inst.term.write(bytes);
}

export function disposeTerminal(elementId: string): void {
  const inst = instances.get(elementId);
  if (!inst) return;
  inst.ro.disconnect();
  inst.term.dispose();
  instances.delete(elementId);
}

export function fitTerminal(elementId: string): void {
  const inst = instances.get(elementId);
  if (!inst) return;
  inst.fitAddon.fit();
}
