import * as vscode from "vscode";
import { BUILTIN_FUNCTIONS, BUILTINS, GLOBAL_FUNCTION_SET } from "./builtins";

const KEYWORD_DOCS: Map<string, string> = new Map([
  ["par", "Execute branches in parallel, returning a tuple of results"],
  ["sel", "Execute branches in parallel, returning the first to complete"],
  ["loop", "Repeat a block until `break` is called"],
  ["break", "Exit the enclosing `loop`, optionally with a value"],
  ["yield", "Yield a value from a generator"],
  ["assert", "Assert a condition is truthy, error if not"],
  ["use", "Import a module or specific bindings from a module"],
  ["Protocol", "Define a message protocol (typed record shape)"],
  ["MCP", "Define an MCP tool connection"],
]);

const OPERATOR_DOCS: Map<string, string> = new Map([
  ["~>", "Send a message to an agent (fire and forget)"],
  ["~>?", "Send a message to an agent and await response"],
  ["??", "Coalesce: use right side if left is None or Err"],
  ["^", "Propagate: unwrap Ok/Some or return Err/None to caller"],
  ["|", "Pipe: pass left side as argument to right side"],
  ["->", "Arrow: used in match arms and function types"],
]);

function formatSignature(entry: { module: string; name: string; arity: number }): string {
  const args = Array.from({ length: entry.arity }, (_, i) => `arg${i + 1}`).join(" ");
  if (entry.module === "global") {
    return args ? `${entry.name} ${args}` : entry.name;
  }
  return args ? `${entry.module}.${entry.name} ${args}` : `${entry.module}.${entry.name}`;
}

export class LxHoverProvider implements vscode.HoverProvider {
  provideHover(
    doc: vscode.TextDocument,
    pos: vscode.Position,
  ): vscode.Hover | undefined {
    const wordRange = doc.getWordRangeAtPosition(pos, /[a-zA-Z_][a-zA-Z0-9_?]*/);
    if (!wordRange) return undefined;
    const word = doc.getText(wordRange);

    const kwDoc = KEYWORD_DOCS.get(word);
    if (kwDoc) {
      const md = new vscode.MarkdownString();
      md.appendCodeblock(word, "lx");
      md.appendText(kwDoc);
      return new vscode.Hover(md, wordRange);
    }

    const charBefore = wordRange.start.character > 0
      ? doc.getText(new vscode.Range(
          pos.line, wordRange.start.character - 1,
          pos.line, wordRange.start.character,
        ))
      : "";

    if (charBefore === ".") {
      const modRange = doc.getWordRangeAtPosition(
        new vscode.Position(pos.line, wordRange.start.character - 2),
        /[a-zA-Z_][a-zA-Z0-9_]*/,
      );
      if (modRange) {
        const modName = doc.getText(modRange);
        const qualified = `${modName}.${word}`;
        const entry = BUILTIN_FUNCTIONS.get(qualified);
        if (entry) {
          const md = new vscode.MarkdownString();
          md.appendCodeblock(formatSignature(entry), "lx");
          md.appendText(entry.description);
          const fullRange = new vscode.Range(modRange.start, wordRange.end);
          return new vscode.Hover(md, fullRange);
        }
      }
    }

    const globalEntry = GLOBAL_FUNCTION_SET.get(word);
    if (globalEntry) {
      const md = new vscode.MarkdownString();
      md.appendCodeblock(formatSignature(globalEntry), "lx");
      md.appendText(globalEntry.description);
      return new vscode.Hover(md, wordRange);
    }

    const moduleEntries = BUILTINS.get(word);
    if (moduleEntries) {
      const md = new vscode.MarkdownString();
      md.appendMarkdown(`**std/${word}** module\n\n`);
      for (const fn of moduleEntries) {
        md.appendMarkdown(`- \`${fn.name}\` — ${fn.description}\n`);
      }
      return new vscode.Hover(md, wordRange);
    }

    return undefined;
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.languages.registerHoverProvider(
      { language: "lx" },
      new LxHoverProvider(),
    ),
  );
}
