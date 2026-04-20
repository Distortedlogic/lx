import type { ExtensionAPI, ExtensionContext, MessageRenderer, SessionEntry } from "@mariozechner/pi-coding-agent";
import { getMarkdownTheme, keyHint } from "@mariozechner/pi-coding-agent";
import { Box, Spacer, Text, truncateToWidth, type Component, visibleWidth } from "@mariozechner/pi-tui";
import { createHash } from "node:crypto";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const baseDir = dirname(fileURLToPath(import.meta.url));
const messageType = "mermaid-local";
const mermaidBlockRe = /```mermaid\s*([\s\S]*?)```/gi;
const collapsedLines = 12;
const maxBlocks = 5;
const maxSourceLines = 400;
const maxSourceChars = 20000;
const asciiPresets = [
  { key: "default", paddingX: 5, boxBorderPadding: 1 },
  { key: "compact", paddingX: 3, boxBorderPadding: 1 },
  { key: "tight", paddingX: 2, boxBorderPadding: 1 },
  { key: "squeezed", paddingX: 1, boxBorderPadding: 0 },
] as const;

type MermaidIssue = { severity: "warning" | "error"; message: string };
type AsciiVariant = { ascii: string; lineCount: number; maxLineWidth: number };
type MermaidDetails = { source: string; ascii: string; variants?: AsciiVariant[]; issues?: MermaidIssue[] };
type MermaidRenderer = (source: string, options: { paddingX: number; boxBorderPadding: number; colorMode: "none" }) => string;

let parseMermaid: ((source: string) => Promise<void>) | null | undefined;
let renderAscii: MermaidRenderer | null | undefined;
let parseLoadError: string | null = null;
let renderLoadError: string | null = null;
let parserWarningShown = false;
let renderWarningShown = false;

function extractText(content: unknown): string {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return "";
  return content
    .map((part: any) => (part && part.type === "text" ? part.text : ""))
    .filter((part: string) => part.trim().length > 0)
    .join("\n");
}

function extractBlocks(text: string, max = Infinity): string[] {
  const blocks: string[] = [];
  mermaidBlockRe.lastIndex = 0;
  let match: RegExpExecArray | null = null;
  while ((match = mermaidBlockRe.exec(text)) !== null) {
    const block = match[1]?.trim();
    if (block) blocks.push(block);
    if (blocks.length >= max) break;
  }
  return blocks;
}

function hash(source: string): string {
  return createHash("sha256").update(source).digest("hex").slice(0, 8);
}

function maxLineWidth(text: string): number {
  return text.split(/\r?\n/).reduce((max, line) => Math.max(max, visibleWidth(line)), 0);
}

function buildContextContent(source: string, issues: MermaidIssue[], includeSource: boolean): string {
  const lines = issues.map((issue) => `[mermaid:${issue.severity}] ${issue.message}`);
  if (!includeSource) return lines.join("\n");
  const block = `\`\`\`mermaid\n%% mermaid-hash: ${hash(source)}\n${source.replace(/\s+$/g, "")}\n\`\`\``;
  return lines.length > 0 ? `${lines.join("\n")}\n\n${block}` : block;
}

async function getParser(): Promise<((source: string) => Promise<void>) | null> {
  if (parseMermaid !== undefined) return parseMermaid;
  try {
    const mod = await import("mermaid");
    const api = (mod as any).default ?? (mod as any).mermaidAPI ?? mod;
    if (typeof api?.initialize === "function") {
      try {
        api.initialize({ startOnLoad: false });
      } catch {}
    }
    if (typeof api?.parse !== "function") {
      parseLoadError = "mermaid parse API not available";
      parseMermaid = null;
      return null;
    }
    parseMermaid = async (source: string) => {
      const result = api.parse(source);
      if (result && typeof result.then === "function") await result;
    };
  } catch (error) {
    parseLoadError = error instanceof Error ? error.message : String(error);
    parseMermaid = null;
  }
  return parseMermaid;
}

async function getAsciiRenderer(): Promise<MermaidRenderer | null> {
  if (renderAscii !== undefined) return renderAscii;
  try {
    const mod = await import("beautiful-mermaid");
    const fn = (mod as any).renderMermaidAscii;
    if (typeof fn !== "function") {
      renderLoadError = "beautiful-mermaid render function not available";
      renderAscii = null;
      return null;
    }
    renderAscii = fn as MermaidRenderer;
  } catch (error) {
    renderLoadError = error instanceof Error ? error.message : String(error);
    renderAscii = null;
  }
  return renderAscii;
}

async function processBlock(source: string, ctx: ExtensionContext): Promise<MermaidDetails> {
  const issues: MermaidIssue[] = [];
  const parser = await getParser();
  if (!parser && !parserWarningShown && ctx.hasUI) {
    ctx.ui.notify(`mermaid-local: parser unavailable (${parseLoadError ?? "unknown error"})`, "warning");
    parserWarningShown = true;
  }
  if (parser) {
    try {
      await parser(source);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (!message.includes("DOMPurify")) {
        issues.push({ severity: "error", message: `Mermaid parse error: ${message}` });
      } else if (!parserWarningShown && ctx.hasUI) {
        ctx.ui.notify(`mermaid-local: parser unavailable (${message})`, "warning");
        parserWarningShown = true;
      }
    }
  }
  const renderer = await getAsciiRenderer();
  if (!renderer) {
    if (!renderWarningShown && ctx.hasUI) {
      ctx.ui.notify(`mermaid-local: renderer unavailable (${renderLoadError ?? "unknown error"})`, "warning");
      renderWarningShown = true;
    }
    issues.push({ severity: "warning", message: `ASCII renderer unavailable: ${renderLoadError ?? "install dependencies in .pi/extensions/mermaid-local"}` });
    return { source, ascii: "[renderer unavailable]", issues };
  }
  const variants = asciiPresets.flatMap((preset) => {
    try {
      const ascii = renderer(source, { paddingX: preset.paddingX, boxBorderPadding: preset.boxBorderPadding, colorMode: "none" }).trimEnd();
      return [{ ascii, lineCount: ascii ? ascii.split(/\r?\n/).length : 0, maxLineWidth: maxLineWidth(ascii) }];
    } catch {
      return [];
    }
  });
  if (variants.length === 0) {
    issues.push({ severity: "error", message: "Mermaid ASCII rendering failed." });
    return { source, ascii: "[render failed]", issues };
  }
  return { source, ascii: variants[0].ascii, variants, issues: issues.length > 0 ? issues : undefined };
}

function selectVariant(width: number, details: MermaidDetails): { ascii: string; lineCount: number; clipped: boolean } {
  const safeWidth = Math.max(1, width);
  const variants = details.variants ?? [{ ascii: details.ascii, lineCount: details.ascii.split(/\r?\n/).length, maxLineWidth: maxLineWidth(details.ascii) }];
  for (const variant of variants) {
    if (variant.maxLineWidth <= safeWidth) return { ascii: variant.ascii, lineCount: variant.lineCount, clipped: false };
  }
  const tightest = variants[variants.length - 1];
  return { ascii: tightest.ascii, lineCount: tightest.lineCount, clipped: tightest.maxLineWidth > safeWidth };
}

function lastAssistant(entries: SessionEntry[]): string | null {
  for (let i = entries.length - 1; i >= 0; i--) {
    const entry = entries[i];
    if (entry.type !== "message" || entry.message.role !== "assistant") continue;
    const text = extractText(entry.message.content);
    if (text.trim()) return text;
  }
  return null;
}

export default function (pi: ExtensionAPI) {
  const renderer: MessageRenderer<MermaidDetails> = (message, { expanded }, theme) => {
    const details = message.details as MermaidDetails | undefined;
    const fallbackSource = extractText(message.content);
    const data = details ?? { source: fallbackSource, ascii: fallbackSource };
    const asciiComponent: Component = {
      render: (width) => {
        const selected = selectVariant(width, data);
        const allLines = selected.ascii.split(/\r?\n/);
        const shown = expanded || allLines.length <= collapsedLines ? allLines : allLines.slice(0, collapsedLines);
        const lines = [truncateToWidth(theme.fg("customMessageLabel", theme.bold("Mermaid (ASCII)")), width)];
        for (const issue of data.issues ?? []) lines.push(truncateToWidth(theme.fg(issue.severity === "error" ? "error" : "warning", issue.message), width));
        for (const line of shown) lines.push(truncateToWidth(line, width, ""));
        if (!expanded && allLines.length > collapsedLines) {
          lines.push(truncateToWidth(theme.fg("muted", `... (${allLines.length - collapsedLines} more lines, ${keyHint("expandTools", "to expand")})`), width));
        }
        if (selected.clipped) lines.push(truncateToWidth(theme.fg("muted", "... (clipped to fit width)"), width));
        return lines;
      },
      invalidate: () => {},
    };
    const box = new Box(1, 1, (text: string) => theme.bg("customMessageBg", text));
    box.addChild(asciiComponent);
    if (expanded && data.source) {
      const markdownTheme = getMarkdownTheme();
      const indent = markdownTheme.codeBlockIndent ?? "  ";
      const highlighted = markdownTheme.highlightCode?.(data.source, "mermaid");
      const codeLines = highlighted ?? data.source.split("\n").map((line) => markdownTheme.codeBlock(line));
      const rendered = [markdownTheme.codeBlockBorder("```mermaid"), ...codeLines.map((line) => `${indent}${line}`), markdownTheme.codeBlockBorder("```")].join("\n");
      box.addChild(new Spacer(1));
      box.addChild(new Text(rendered, 0, 0));
    }
    return box;
  };

  const renderBlocks = async (blocks: string[], ctx: ExtensionContext, includeSource: boolean) => {
    if (blocks.length > maxBlocks && ctx.hasUI) ctx.ui.notify(`mermaid-local: rendering first ${maxBlocks} Mermaid blocks.`, "warning");
    for (const [index, block] of blocks.slice(0, maxBlocks).entries()) {
      const lineCount = block.split(/\r?\n/).length;
      if (lineCount > maxSourceLines || block.length > maxSourceChars) {
        if (ctx.hasUI) ctx.ui.notify(`mermaid-local: skipped Mermaid block ${index + 1}; it is too large.`, "warning");
        continue;
      }
      const details = await processBlock(block, ctx);
      pi.sendMessage({ customType: messageType, content: buildContextContent(block, details.issues ?? [], includeSource), display: true, details });
    }
  };

  pi.on("resources_discover", () => ({ skillPaths: [join(baseDir, "skills")] }));
  pi.registerMessageRenderer(messageType, renderer);
  pi.on("input", async (event, ctx) => {
    if (event.source === "extension") return { action: "continue" };
    const text = typeof event.text === "string" ? event.text : "";
    const blocks = extractBlocks(text, maxBlocks + 1);
    if (blocks.length === 0) return { action: "continue" };
    await renderBlocks(blocks, ctx, blocks.length > 1);
    return { action: "continue" };
  });
  pi.on("agent_end", async (event, ctx) => {
    for (let i = event.messages.length - 1; i >= 0; i--) {
      const message = event.messages[i];
      if (message.role !== "assistant") continue;
      const blocks = extractBlocks(extractText(message.content), maxBlocks + 1);
      if (blocks.length > 0) {
        await renderBlocks(blocks, ctx, blocks.length > 1);
        return;
      }
    }
  });
  pi.registerCommand("mermaid-local", {
    description: "Render Mermaid in the last assistant message as ASCII",
    handler: async (_args, ctx) => {
      const text = lastAssistant(ctx.sessionManager.getBranch());
      if (!text) {
        if (ctx.hasUI) ctx.ui.notify("mermaid-local: no assistant message found", "warning");
        return;
      }
      const blocks = extractBlocks(text, maxBlocks + 1);
      if (blocks.length === 0) {
        if (ctx.hasUI) ctx.ui.notify("mermaid-local: no Mermaid blocks found", "warning");
        return;
      }
      await renderBlocks(blocks, ctx, true);
    },
  });
}
