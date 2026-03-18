import * as vscode from "vscode";
import { execFile } from "child_process";
import { basename, dirname, join } from "path";

let log: vscode.OutputChannel;

function lxBinary(): string {
  const configured = vscode.workspace
    .getConfiguration("lx.diagram")
    .get<string>("binaryPath", "");
  return configured || "lx";
}

function mmdPath(lxPath: string): string {
  const dir = dirname(lxPath);
  const name = basename(lxPath, ".lx");
  return join(dir, `${name}.mmd`);
}

function generateDiagram(filePath: string): void {
  const bin = lxBinary();
  const out = mmdPath(filePath);
  log.appendLine(`running: ${bin} diagram ${filePath} -o ${out}`);
  execFile(bin, ["diagram", filePath, "-o", out], (err, stdout, stderr) => {
    if (err) {
      log.appendLine(`error: ${stderr || err.message}`);
      vscode.window.showWarningMessage(`lx diagram: ${stderr || err.message}`);
    } else {
      log.appendLine(`wrote ${out}`);
      if (stdout) log.appendLine(stdout);
    }
  });
}

export function activate(ctx: vscode.ExtensionContext): void {
  log = vscode.window.createOutputChannel("lx");
  log.appendLine("lx extension activated");

  ctx.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId !== "lx") return;
      const enabled = vscode.workspace
        .getConfiguration("lx.diagram")
        .get<boolean>("autoGenerate", true);
      if (!enabled) return;
      generateDiagram(doc.uri.fsPath);
    })
  );

  ctx.subscriptions.push(
    vscode.commands.registerCommand("lx.generateDiagram", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "lx") {
        vscode.window.showWarningMessage("Open a .lx file first");
        return;
      }
      generateDiagram(editor.document.uri.fsPath);
    })
  );
}

export function deactivate(): void {}
