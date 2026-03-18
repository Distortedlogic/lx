import * as vscode from "vscode";

let item: vscode.StatusBarItem;

export function update(errorCount: number): void {
  if (errorCount === 0) {
    item.text = "$(check) lx";
    item.tooltip = "lx: no errors";
  } else {
    item.text = `$(warning) lx: ${errorCount}`;
    item.tooltip = `lx: ${errorCount} error(s)`;
  }
}

function syncVisibility(): void {
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document.languageId === "lx") {
    item.show();
  } else {
    item.hide();
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  item = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  item.command = "lx.runFile";
  item.text = "$(check) lx";
  item.tooltip = "lx: no errors";
  ctx.subscriptions.push(item);

  syncVisibility();
  ctx.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(() => syncVisibility()),
  );
}
