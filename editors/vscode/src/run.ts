import * as vscode from "vscode";
import { lxBinary } from "./diagram";

let terminal: vscode.Terminal | undefined;

function getTerminal(): vscode.Terminal {
  if (terminal && !terminal.exitStatus) return terminal;
  terminal = vscode.window.createTerminal("lx");
  return terminal;
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.window.onDidCloseTerminal((closed) => {
      if (closed === terminal) terminal = undefined;
    }),
  );

  ctx.subscriptions.push(
    vscode.commands.registerCommand("lx.runFile", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "lx") {
        vscode.window.showWarningMessage("Open a .lx file first");
        return;
      }
      const t = getTerminal();
      t.show();
      t.sendText(`${lxBinary()} run "${editor.document.uri.fsPath}"`);
    }),
  );
}
