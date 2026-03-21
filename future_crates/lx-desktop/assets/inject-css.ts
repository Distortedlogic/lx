import xtermCss from "@xterm/xterm/css/xterm.css?inline";

let injected = false;

export function ensureXtermCss(): void {
  if (injected) return;
  injected = true;
  const style = document.createElement("style");
  style.textContent = xtermCss;
  document.head.appendChild(style);
}
