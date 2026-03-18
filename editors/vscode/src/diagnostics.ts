import * as vscode from "vscode";
import { execFile } from "child_process";
import { lxBinary } from "./diagram";

const LOCATION_RE = /╭─\[(.+?):(\d+):(\d+)\]/;
const MESSAGE_RE = /×\s*(.+)/;

interface ParsedDiagnostic {
  line: number;
  col: number;
  message: string;
}

function parseMietteOutput(stderr: string): ParsedDiagnostic[] {
  const results: ParsedDiagnostic[] = [];
  const lines = stderr.split("\n");

  let pendingMessage: string | undefined;

  for (const line of lines) {
    const msgMatch = line.match(MESSAGE_RE);
    if (msgMatch) {
      pendingMessage = msgMatch[1].trim();
      continue;
    }

    const locMatch = line.match(LOCATION_RE);
    if (locMatch && pendingMessage) {
      results.push({
        line: parseInt(locMatch[2], 10) - 1,
        col: parseInt(locMatch[3], 10) - 1,
        message: pendingMessage,
      });
      pendingMessage = undefined;
    }
  }

  return results;
}

let collection: vscode.DiagnosticCollection;
let log: vscode.OutputChannel;
let onErrorCount: (count: number) => void;
let binaryMissing = false;

function runCheck(doc: vscode.TextDocument): void {
  if (binaryMissing) return;

  const bin = lxBinary();
  const filePath = doc.uri.fsPath;

  execFile(bin, ["check", filePath], (err, _stdout, stderr) => {
    if (err && (err as NodeJS.ErrnoException).code === "ENOENT") {
      log.appendLine(`lx binary not found at "${bin}" — diagnostics disabled`);
      binaryMissing = true;
      return;
    }

    const parsed = parseMietteOutput(stderr || "");
    const diagnostics: vscode.Diagnostic[] = parsed.map((d) => {
      const line = Math.max(0, d.line);
      const col = Math.max(0, d.col);
      const lineEnd = doc.lineCount > line ? doc.lineAt(line).text.length : col + 1;
      const range = new vscode.Range(line, col, line, lineEnd);
      return new vscode.Diagnostic(range, d.message, vscode.DiagnosticSeverity.Error);
    });

    collection.set(doc.uri, diagnostics);
    onErrorCount(diagnostics.length);
  });
}

export function activate(
  ctx: vscode.ExtensionContext,
  outputChannel: vscode.OutputChannel,
  errorCountCallback: (count: number) => void,
): void {
  log = outputChannel;
  onErrorCount = errorCountCallback;
  collection = vscode.languages.createDiagnosticCollection("lx");
  ctx.subscriptions.push(collection);

  ctx.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId !== "lx") return;
      const enabled = vscode.workspace
        .getConfiguration("lx.diagnostics")
        .get<boolean>("onSave", true);
      if (!enabled) return;
      runCheck(doc);
    }),
  );

  ctx.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((doc) => {
      collection.delete(doc.uri);
    }),
  );

  ctx.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("lx.binaryPath") || e.affectsConfiguration("lx.diagram.binaryPath")) {
        binaryMissing = false;
      }
    }),
  );
}
