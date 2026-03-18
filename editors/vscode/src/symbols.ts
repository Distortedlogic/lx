import * as vscode from "vscode";

const BINDING_RE = /^(\+?\w+)\s*=/;
const PROTOCOL_RE = /^Protocol\s+(\w+)/;
const USE_RE = /^use\s+(.+)/;

function findBlockEnd(
  doc: vscode.TextDocument,
  startLine: number,
): number {
  let depth = 0;
  let foundOpen = false;
  for (let i = startLine; i < doc.lineCount; i++) {
    const text = doc.lineAt(i).text;
    for (const ch of text) {
      if (ch === "{") {
        depth++;
        foundOpen = true;
      }
      if (ch === "}") depth--;
    }
    if (foundOpen && depth <= 0) return i;
  }
  return startLine;
}

function findNextToplevel(
  doc: vscode.TextDocument,
  afterLine: number,
): number {
  for (let i = afterLine + 1; i < doc.lineCount; i++) {
    const text = doc.lineAt(i).text;
    if (text.length === 0 || text.startsWith("--")) continue;
    if (/^\S/.test(text)) return i - 1;
  }
  return doc.lineCount - 1;
}

function isTaggedUnion(doc: vscode.TextDocument, line: number): boolean {
  if (line + 1 >= doc.lineCount) return false;
  return /^\s*\|/.test(doc.lineAt(line + 1).text);
}

export class LxDocumentSymbolProvider
  implements vscode.DocumentSymbolProvider
{
  provideDocumentSymbols(
    doc: vscode.TextDocument,
  ): vscode.DocumentSymbol[] {
    const symbols: vscode.DocumentSymbol[] = [];

    for (let i = 0; i < doc.lineCount; i++) {
      const line = doc.lineAt(i);
      const text = line.text;

      const protoMatch = text.match(PROTOCOL_RE);
      if (protoMatch) {
        const name = protoMatch[1];
        const endLine = text.includes("{")
          ? findBlockEnd(doc, i)
          : i;
        const range = new vscode.Range(i, 0, endLine, doc.lineAt(endLine).text.length);
        symbols.push(
          new vscode.DocumentSymbol(
            name,
            "Protocol",
            vscode.SymbolKind.Interface,
            range,
            new vscode.Range(i, text.indexOf(name), i, text.indexOf(name) + name.length),
          ),
        );
        continue;
      }

      const useMatch = text.match(USE_RE);
      if (useMatch) {
        const path = useMatch[1].trim();
        const range = new vscode.Range(i, 0, i, text.length);
        symbols.push(
          new vscode.DocumentSymbol(
            path,
            "import",
            vscode.SymbolKind.Module,
            range,
            range,
          ),
        );
        continue;
      }

      const bindMatch = text.match(BINDING_RE);
      if (bindMatch) {
        const name = bindMatch[1];

        if (isTaggedUnion(doc, i)) {
          let endLine = i + 1;
          while (endLine + 1 < doc.lineCount && /^\s*\|/.test(doc.lineAt(endLine + 1).text)) {
            endLine++;
          }
          const range = new vscode.Range(i, 0, endLine, doc.lineAt(endLine).text.length);
          symbols.push(
            new vscode.DocumentSymbol(
              name,
              "type",
              vscode.SymbolKind.Enum,
              range,
              new vscode.Range(i, 0, i, name.length),
            ),
          );
          i = endLine;
          continue;
        }

        const hasBlock = text.includes("{");
        const endLine = hasBlock
          ? findBlockEnd(doc, i)
          : findNextToplevel(doc, i);
        const range = new vscode.Range(i, 0, endLine, doc.lineAt(endLine).text.length);
        const kind = name.startsWith("+")
          ? vscode.SymbolKind.Function
          : vscode.SymbolKind.Variable;
        const detail = name.startsWith("+") ? "export" : "";
        symbols.push(
          new vscode.DocumentSymbol(
            name,
            detail,
            kind,
            range,
            new vscode.Range(i, 0, i, name.length),
          ),
        );
        continue;
      }
    }

    return symbols;
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider(
      { language: "lx" },
      new LxDocumentSymbolProvider(),
    ),
  );
}
