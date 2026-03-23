import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";
import {
  mountEditor,
  updateEditor,
  resizeEditor,
  disposeEditor,
} from "../src/editor";

const editorWidget: Widget = {
  mount(elementId: string, config: unknown, dx) {
    mountEditor(
      elementId,
      config as { content?: string; language?: string; filePath?: string },
      dx,
    );
  },

  update(elementId: string, data: unknown) {
    updateEditor(elementId, (data as { content?: string }).content ?? "");
  },

  resize(elementId: string) {
    resizeEditor(elementId);
  },

  dispose(elementId: string) {
    disposeEditor(elementId);
  },
};

registerWidget("editor", editorWidget);
