import * as vscode from "vscode";
import { GLOBAL_FUNCTION_SET } from "./builtins";

const TOKEN_TYPES = [
  "comment",
  "string",
  "keyword",
  "number",
  "operator",
  "function",
  "variable",
  "parameter",
  "type",
  "namespace",
  "enumMember",
  "property",
];

const TOKEN_MODIFIERS = [
  "declaration",
  "definition",
  "readonly",
  "defaultLibrary",
];

export const LEGEND = new vscode.SemanticTokensLegend(TOKEN_TYPES, TOKEN_MODIFIERS);

const KEYWORDS = new Set([
  "par", "sel", "loop", "break", "yield", "assert", "use", "Protocol", "MCP",
  "true", "false", "None",
]);

const COMMENT_IDX = 0;
const KEYWORD_IDX = 2;
const NUMBER_IDX = 3;
const FUNCTION_IDX = 5;
const VARIABLE_IDX = 6;
const TYPE_IDX = 8;

const DECLARATION_BIT = 1 << 0;
const DEFAULT_LIB_BIT = 1 << 3;

const IDENT_RE = /[a-zA-Z_][a-zA-Z0-9_?]*/g;
const NUMBER_RE = /\b(?:0x[0-9a-fA-F_]+|0b[01_]+|0o[0-7_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?)\b/g;
const BINDING_RE = /^(\+?[a-zA-Z_]\w*)\s*(?:=|:=)/;

export class LxSemanticTokensProvider implements vscode.DocumentSemanticTokensProvider {
  provideDocumentSemanticTokens(
    doc: vscode.TextDocument,
  ): vscode.SemanticTokens {
    const builder = new vscode.SemanticTokensBuilder(LEGEND);

    for (let i = 0; i < doc.lineCount; i++) {
      const text = doc.lineAt(i).text;

      if (/^\s*--/.test(text)) {
        builder.push(new vscode.Range(i, 0, i, text.length), TOKEN_TYPES[COMMENT_IDX]);
        continue;
      }

      const bindMatch = text.match(BINDING_RE);
      if (bindMatch) {
        const name = bindMatch[1];
        const start = text.indexOf(name);
        builder.push(
          new vscode.Range(i, start, i, start + name.length),
          TOKEN_TYPES[VARIABLE_IDX],
          [TOKEN_MODIFIERS[0]],
        );
      }

      IDENT_RE.lastIndex = 0;
      let m: RegExpExecArray | null;
      while ((m = IDENT_RE.exec(text)) !== null) {
        const word = m[0];
        const col = m.index;

        if (KEYWORDS.has(word)) {
          builder.push(new vscode.Range(i, col, i, col + word.length), TOKEN_TYPES[KEYWORD_IDX]);
        } else if (GLOBAL_FUNCTION_SET.has(word)) {
          builder.push(
            new vscode.Range(i, col, i, col + word.length),
            TOKEN_TYPES[FUNCTION_IDX],
            [TOKEN_MODIFIERS[3]],
          );
        } else if (/^[A-Z]/.test(word)) {
          builder.push(new vscode.Range(i, col, i, col + word.length), TOKEN_TYPES[TYPE_IDX]);
        }
      }

      NUMBER_RE.lastIndex = 0;
      while ((m = NUMBER_RE.exec(text)) !== null) {
        builder.push(
          new vscode.Range(i, m.index, i, m.index + m[0].length),
          TOKEN_TYPES[NUMBER_IDX],
        );
      }
    }

    return builder.build();
  }
}

export function activate(ctx: vscode.ExtensionContext): void {
  ctx.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      { language: "lx" },
      new LxSemanticTokensProvider(),
      LEGEND,
    ),
  );
}
