import { EditorView, keymap } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { basicSetup } from "codemirror";
import { oneDark } from "@codemirror/theme-one-dark";
import { javascript } from "@codemirror/lang-javascript";
import { python } from "@codemirror/lang-python";
import { json } from "@codemirror/lang-json";
import { html } from "@codemirror/lang-html";
import { css } from "@codemirror/lang-css";
import { rust } from "@codemirror/lang-rust";
import type { Dioxus } from "./types";

const instances = new Map<string, EditorView>();

function detectLanguage(filePath?: string) {
  if (!filePath) return null;
  const ext = filePath.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "js":
    case "mjs":
    case "cjs":
      return javascript();
    case "ts":
    case "mts":
    case "cts":
      return javascript({ typescript: true });
    case "jsx":
      return javascript({ jsx: true });
    case "tsx":
      return javascript({ jsx: true, typescript: true });
    case "py":
      return python();
    case "json":
      return json();
    case "html":
    case "htm":
      return html();
    case "css":
      return css();
    case "rs":
      return rust();
    default:
      return null;
  }
}

export function mountEditor(
  elementId: string,
  config: { content?: string; language?: string; filePath?: string },
  dx: Dioxus,
): void {
  const lang = detectLanguage(config.filePath);
  const extensions = [
    basicSetup,
    oneDark,
    EditorView.theme({
      "&": { height: "100%" },
      ".cm-scroller": { overflow: "auto" },
    }),
    keymap.of([
      {
        key: "Mod-s",
        run: (view) => {
          dx.send({ type: "save", content: view.state.doc.toString() });
          return true;
        },
      },
    ]),
  ];

  if (lang) {
    extensions.push(lang);
  }

  const state = EditorState.create({
    doc: config.content ?? "",
    extensions,
  });

  const parent = document.getElementById(elementId);
  if (!parent) throw new Error(`editor container not found: ${elementId}`);

  const view = new EditorView({ state, parent });
  instances.set(elementId, view);
}

export function updateEditor(elementId: string, content: string): void {
  const view = instances.get(elementId);
  if (!view) return;
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: content },
  });
}

export function resizeEditor(elementId: string): void {
  const view = instances.get(elementId);
  if (!view) return;
  view.requestMeasure();
}

export function disposeEditor(elementId: string): void {
  const view = instances.get(elementId);
  if (!view) return;
  view.destroy();
  instances.delete(elementId);
}
