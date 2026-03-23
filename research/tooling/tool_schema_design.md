# Tool Calling Best Practices

How to design tools/functions that LLMs use effectively, from schema design to error handling to token efficiency.

## How Tool Calling Works

1. **Definition**: Developer provides tool schemas (name, description, parameters) in the system prompt
2. **Selection**: LLM analyzes user intent and selects appropriate tool(s)
3. **Invocation**: LLM generates structured JSON describing the call (does NOT execute it)
4. **Execution**: Host application deserializes and executes the function
5. **Feedback**: Result returned to LLM for reasoning about next step
6. **Loop**: LLM decides whether to call another tool or produce final response

## Tool Schema Design

### Name

- Use `snake_case` with verb-first naming: `search_code`, `create_task`, `get_file`
- Be specific: `get_user_by_email` not `get_user`
- Keep names short but descriptive

### Description

The **most critical field**. Must include:

- **What** the tool does
- **When** to use it (and when NOT to)
- **Capability boundaries** (what it can't do)
- **Return format** description

Think of descriptions like docstrings for a junior developer. The model needs enough context to decide if and when to use the tool.

### Parameters

- Use descriptive parameter names: `file_path` not `p`
- Constrain with JSON Schema: enums, ranges, patterns, required fields
- Apply **poka-yoke** (mistake-proofing): use absolute paths instead of relative, enums instead of free-text where possible
- Reduce optionality: fewer optional params = fewer mistakes

### Real-World Example (Anthropic SWE-bench)

Switching tool parameters from relative to absolute file paths eliminated an entire class of model errors -- "the model used this method flawlessly."

## Tool Granularity

### Atomic & Composable (Preferred)

Each tool does **one thing well**. Let the agent orchestrate sequences:

```
read_file(path) → search_code(query) → edit_file(path, changes)
```

Advantages: reusable, testable, debuggable, predictable.

### Avoid Over-Mapping

Do NOT map every API endpoint to a new MCP tool. Group related operations into higher-level functions where it makes sense. Balance between:

- Too fine-grained: agent needs many calls for simple tasks
- Too coarse: tool does too much, hard for agent to use correctly

### Namespace Grouping

Group related tools logically:
- `climate.read_temperature`, `climate.set_target_temp`
- `task.create`, `task.list`, `task.update`

Improves discoverability and organization.

## Tool Annotations (MCP)

MCP defines behavioral hints that help agents and UIs make safer decisions:

| Annotation | Purpose | Example |
|---|---|---|
| `readOnlyHint` | Tool doesn't modify environment | `search_code`, `list_files` |
| `destructiveHint` | Tool may perform destructive updates | `delete_file`, `drop_table` |
| `idempotentHint` | Repeated calls have no additional effect | `set_config`, `update_status` |
| `openWorldHint` | Tool interacts with external entities | `web_search`, `send_email` |

Rules:
- `destructiveHint` and `idempotentHint` only meaningful when `readOnlyHint` is false
- These are **hints**, not guarantees -- guidance for clients, not enforcement

## Tool Result Design

### What to Include

- Structured data the LLM can reason about (JSON, formatted text)
- Enough context for the LLM to decide next steps
- Error information with actionable details

### What to Exclude

- Massive outputs that flood the context window
- Binary data the LLM can't interpret
- Implementation details the LLM doesn't need

### Error Results

Return structured errors with:
- Error type/code
- Human-readable message
- Actionable suggestion for recovery
- Never silently fail -- explicit failures let the agent recover or escalate

## Token Efficiency

### The Scaling Problem

Tool definitions consume tokens on **every LLM call**:
- 58 tools can consume ~55K tokens (Anthropic internal testing)
- As tool count increases, model accuracy in selecting the correct tool **decreases**

### Mitigation Strategies

1. **Tool discovery/registry**: Query relevant tools based on user intent rather than loading all
2. **Concise descriptions**: Be precise, not verbose. Every word costs tokens
3. **Lazy loading**: Only include tools relevant to current task phase
4. **Result compression**: Summarize large tool outputs before returning to LLM
5. **Avoid routing large outputs through the model**: Store externally, return references

## Sequential vs Parallel Tool Calls

### Sequential

Agent calls tools one at a time, using each result to decide the next call. Default behavior for most agent loops.

### Parallel

Agent identifies independent tool calls and issues them simultaneously. Requirements:
- No dependencies between the calls
- Results don't inform each other's parameters

Reduces latency significantly for independent operations (e.g., searching multiple sources simultaneously).

## Security

- **Restrict action space**: Use explicit conditional logic, never `eval()` or dynamic invocation
- **Input sanitization**: Validate all parameters against schemas before execution
- **Principle of least privilege**: Tools should have minimal required permissions
- **Audit logging**: Log every invocation with inputs, outputs, and errors

## Key Principles

1. **Invest in tool design proportional to HCI** -- tool descriptions are the "interface" for the LLM
2. **Test extensively** -- use workbenches to identify model mistakes and iterate on descriptions
3. **Format matters** -- different specifications for identical actions have vastly different difficulty levels for LLMs
4. **Give tokens to think** -- allow enough output space before the model commits to a structure
5. **Keep formats natural** -- close to what appears in internet training data

## References

- [Function Calling using LLMs (Martin Fowler)](https://martinfowler.com/articles/function-call-LLM.html)
- [Tool Calling Guide 2026 (Composio)](https://composio.dev/blog/ai-agent-tool-calling-guide)
- [Function Calling in LLM Agents (Symflower)](https://symflower.com/en/company/blog/2025/function-calling-llm-agents/)
- [Advanced Tool Calling in LLM Agents (SparkCo)](https://sparkco.ai/blog/advanced-tool-calling-in-llm-agents-a-deep-dive)
- [Function Calling in AI Agents (Prompting Guide)](https://www.promptingguide.ai/agents/function-calling)
- [Mastering LLM Tool Calling (ML Mastery)](https://machinelearningmastery.com/mastering-llm-tool-calling-the-complete-framework-for-connecting-models-to-the-real-world/)
- [Tools - MCP Specification](https://modelcontextprotocol.io/legacy/concepts/tools)
- [Agent Workflows and Tool Design for Edge MCP Servers (Glama)](https://glama.ai/blog/2025-08-22-agent-workflows-and-tool-design-for-edge-mcp-servers)
- [MCP Tool Annotations (Marc Nuri)](https://blog.marcnuri.com/mcp-tool-annotations-introduction)
