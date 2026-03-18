import * as vscode from "vscode";
import * as diagram from "./diagram";
import * as run from "./run";
import * as statusbar from "./statusbar";
import * as symbols from "./symbols";
import * as hover from "./hover";
import * as diagnostics from "./diagnostics";
import * as codelens from "./codelens";
import * as tasks from "./tasks";
import * as semantic from "./semantic";

export function activate(ctx: vscode.ExtensionContext): void {
  const log = vscode.window.createOutputChannel("lx");
  log.appendLine("lx extension activated");

  diagram.activate(ctx, log);
  run.activate(ctx);
  statusbar.activate(ctx);
  symbols.activate(ctx);
  hover.activate(ctx);
  diagnostics.activate(ctx, log, statusbar.update);
  codelens.activate(ctx);
  tasks.activate(ctx);
  semantic.activate(ctx);
}

export function deactivate(): void {}
