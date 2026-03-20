import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";

const terminals: Map<string, { term: Terminal; fit: FitAddon }> = new Map();

export function createTerminal(
  containerId: string,
  paneId: string,
  wsUrl: string,
): void {
  const container = document.getElementById(containerId);
  if (!container) return;

  const term = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: "monospace",
    theme: { background: "#1a1a2e" },
  });

  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open(container);
  fit.fit();

  const ws = new WebSocket(wsUrl);
  ws.binaryType = "arraybuffer";

  ws.onmessage = (event: MessageEvent) => {
    const data =
      event.data instanceof ArrayBuffer
        ? new TextDecoder().decode(event.data)
        : event.data;
    term.write(data);
  };

  term.onData((data: string) => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(data);
    }
  });

  terminals.set(paneId, { term, fit });
}

export function resizeTerminal(paneId: string): void {
  const entry = terminals.get(paneId);
  if (entry) {
    entry.fit.fit();
  }
}

export function writeToTerminal(paneId: string, data: string): void {
  const entry = terminals.get(paneId);
  if (entry) {
    entry.term.write(data);
  }
}

export function destroyTerminal(paneId: string): void {
  const entry = terminals.get(paneId);
  if (entry) {
    entry.term.dispose();
    terminals.delete(paneId);
  }
}
