import * as vscode from "vscode";

const ENTRY_RE = /^\+\w+\s*=/;

export class LxCodeLensProvider implements vscode.CodeLensProvider {
  provideCodeLenses(doc: vscode.TextDocument): vscode.CodeLens[] {
    const lenses: vscode.CodeLens[] = [];

    for (let i = 0; i < doc.lineCount; i++) {
      const text = doc.lineAt(i).text;
      if (ENTRY_RE.test(text)) {
        const range = new vscode.Range(i, 0, i, 0);
        lenses.push(
          new vscode.CodeLens(range, {
            title: "$(play) Run",
            command: "lx.runFile",
            tooltip: "Run this lx file",
          }),
        );
      }
    }

    return lenses;
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      { language: "lx" },
      new LxCodeLensProvider(),
    ),
  );
}
