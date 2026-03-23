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
    fontSize: 13,
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    lineHeight: 1.3,
    theme: {
      background: "#0A0A10",
      foreground: "#D4D4D4",
      cursor: "#FFD600",
      cursorAccent: "#0A0A10",
      selectionBackground: "rgba(255,214,0,0.2)",
      selectionForeground: "#FFFFFF",
      black: "#1A1A22",
      red: "#F44336",
      green: "#4CAF50",
      yellow: "#FFD600",
      blue: "#42A5F5",
      magenta: "#CE93D8",
      cyan: "#26C6DA",
      white: "#E0E0E0",
      brightBlack: "#555565",
      brightRed: "#EF5350",
      brightGreen: "#66BB6A",
      brightYellow: "#FFEB3B",
      brightBlue: "#90CAF9",
      brightMagenta: "#CE93D8",
      brightCyan: "#4DD0E1",
      brightWhite: "#FFFFFF",
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
