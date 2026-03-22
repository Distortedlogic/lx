import type { Dioxus } from "../src/types";
import type { Widget } from "../src/registry";
import { registerWidget } from "../src/registry";

interface AgentState {
  messagesDiv: HTMLDivElement;
  textarea: HTMLTextAreaElement;
  currentBubble: HTMLDivElement | null;
  currentText: string;
  userScrolled: boolean;
  dx: Dioxus;
}

const states = new Map<string, AgentState>();

function createBubble(
  messagesDiv: HTMLDivElement,
  role: "assistant" | "user" | "tool" | "error",
): HTMLDivElement {
  const bubble = document.createElement("div");
  bubble.style.padding = "8px 12px";
  bubble.style.borderRadius = "6px";
  bubble.style.maxWidth = "85%";
  bubble.style.lineHeight = "1.5";
  bubble.style.fontSize = "14px";
  bubble.style.wordWrap = "break-word";

  if (role === "user") {
    bubble.style.alignSelf = "flex-end";
    bubble.style.background = "#2a2a3e";
    bubble.style.color = "#e0e0e0";
  } else if (role === "assistant") {
    bubble.style.alignSelf = "flex-start";
    bubble.style.background = "#1a1a2e";
    bubble.style.color = "#e0e0e0";
  } else if (role === "tool") {
    bubble.style.alignSelf = "flex-start";
    bubble.style.background = "#1a1a2e";
    bubble.style.color = "#e0e0e0";
    bubble.style.border = "1px solid #444";
  } else if (role === "error") {
    bubble.style.alignSelf = "flex-start";
    bubble.style.background = "rgba(220, 38, 38, 0.15)";
    bubble.style.color = "#fca5a5";
  }

  messagesDiv.appendChild(bubble);
  return bubble;
}

function autoScroll(state: AgentState) {
  if (!state.userScrolled) {
    state.messagesDiv.scrollTop = state.messagesDiv.scrollHeight;
  }
}

const agentWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    const el = document.getElementById(elementId);
    if (!el) return;

    const container = document.createElement("div");
    container.style.display = "flex";
    container.style.flexDirection = "column";
    container.style.height = "100%";

    const messagesDiv = document.createElement("div");
    messagesDiv.style.flex = "1";
    messagesDiv.style.overflowY = "auto";
    messagesDiv.style.padding = "16px";
    messagesDiv.style.display = "flex";
    messagesDiv.style.flexDirection = "column";
    messagesDiv.style.gap = "12px";

    const inputBar = document.createElement("div");
    inputBar.style.display = "flex";
    inputBar.style.padding = "8px";
    inputBar.style.gap = "8px";
    inputBar.style.borderTop = "1px solid #444";
    inputBar.style.background = "#1a1a2e";

    const textarea = document.createElement("textarea");
    textarea.style.flex = "1";
    textarea.style.resize = "none";
    textarea.style.background = "#0a0a0a";
    textarea.style.border = "1px solid #444";
    textarea.style.color = "#e0e0e0";
    textarea.style.padding = "8px";
    textarea.style.borderRadius = "4px";
    textarea.style.fontSize = "14px";
    textarea.rows = 1;

    const sendBtn = document.createElement("button");
    sendBtn.textContent = "Send";
    sendBtn.style.background = "#3b82f6";
    sendBtn.style.padding = "8px 16px";
    sendBtn.style.borderRadius = "4px";
    sendBtn.style.fontWeight = "600";
    sendBtn.style.border = "none";
    sendBtn.style.color = "#fff";
    sendBtn.style.cursor = "pointer";

    inputBar.appendChild(textarea);
    inputBar.appendChild(sendBtn);
    container.appendChild(messagesDiv);
    container.appendChild(inputBar);
    el.appendChild(container);

    const state: AgentState = {
      messagesDiv,
      textarea,
      currentBubble: null,
      currentText: "",
      userScrolled: false,
      dx,
    };
    states.set(elementId, state);

    messagesDiv.addEventListener("scroll", () => {
      const atBottom =
        messagesDiv.scrollHeight - messagesDiv.scrollTop - messagesDiv.clientHeight < 30;
      state.userScrolled = !atBottom;
    });

    const sendMessage = () => {
      const content = textarea.value.trim();
      if (!content) return;
      const bubble = createBubble(messagesDiv, "user");
      bubble.textContent = content;
      dx.send({ type: "user_message", content });
      textarea.value = "";
      state.userScrolled = false;
      autoScroll(state);
    };

    sendBtn.addEventListener("click", sendMessage);
    textarea.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && e.ctrlKey) {
        e.preventDefault();
        sendMessage();
      }
    });
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;

    const msg = data as {
      type: string;
      text?: string;
      call_id?: string;
      name?: string;
      arguments?: string;
      message?: string;
    };

    switch (msg.type) {
      case "assistant_chunk": {
        if (!state.currentBubble) {
          state.currentBubble = createBubble(state.messagesDiv, "assistant");
          state.currentText = "";
        }
        state.currentText += msg.text ?? "";
        state.currentBubble.textContent = state.currentText;
        autoScroll(state);
        break;
      }
      case "assistant_done": {
        state.currentBubble = null;
        state.currentText = "";
        break;
      }
      case "tool_call": {
        const bubble = createBubble(state.messagesDiv, "tool");
        const title = document.createElement("div");
        title.style.fontWeight = "600";
        title.style.marginBottom = "4px";
        title.textContent = `Tool: ${msg.name ?? "unknown"}`;
        bubble.appendChild(title);

        if (msg.arguments) {
          const args = document.createElement("pre");
          args.style.fontSize = "12px";
          args.style.margin = "4px 0";
          args.style.whiteSpace = "pre-wrap";
          args.textContent = msg.arguments;
          bubble.appendChild(args);
        }

        const btnRow = document.createElement("div");
        btnRow.style.display = "flex";
        btnRow.style.gap = "8px";
        btnRow.style.marginTop = "8px";

        const approveBtn = document.createElement("button");
        approveBtn.textContent = "Approve";
        approveBtn.style.padding = "4px 12px";
        approveBtn.style.borderRadius = "4px";
        approveBtn.style.border = "1px solid #444";
        approveBtn.style.background = "#2a2a3e";
        approveBtn.style.color = "#e0e0e0";
        approveBtn.style.cursor = "pointer";

        const denyBtn = document.createElement("button");
        denyBtn.textContent = "Deny";
        denyBtn.style.padding = "4px 12px";
        denyBtn.style.borderRadius = "4px";
        denyBtn.style.border = "1px solid #444";
        denyBtn.style.background = "#2a2a3e";
        denyBtn.style.color = "#e0e0e0";
        denyBtn.style.cursor = "pointer";

        approveBtn.addEventListener("click", () => {
          state.dx.send({ type: "tool_decision", call_id: msg.call_id, decision: "approve" });
          approveBtn.disabled = true;
          denyBtn.disabled = true;
          btnRow.style.opacity = "0.5";
        });

        denyBtn.addEventListener("click", () => {
          state.dx.send({ type: "tool_decision", call_id: msg.call_id, decision: "deny" });
          approveBtn.disabled = true;
          denyBtn.disabled = true;
          btnRow.style.opacity = "0.5";
        });

        btnRow.appendChild(approveBtn);
        btnRow.appendChild(denyBtn);
        bubble.appendChild(btnRow);
        autoScroll(state);
        break;
      }
      case "error": {
        const bubble = createBubble(state.messagesDiv, "error");
        bubble.textContent = msg.message ?? "Unknown error";
        autoScroll(state);
        break;
      }
    }
  },

  resize(_elementId: string) {},

  dispose(elementId: string) {
    states.delete(elementId);
    const el = document.getElementById(elementId);
    if (el) el.innerHTML = "";
  },
};

registerWidget("agent", agentWidget);
