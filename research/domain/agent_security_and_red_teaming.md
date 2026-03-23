# Agent Security, Adversarial Attacks, and Red Teaming

## 1. Prompt Injection Attacks on Agents

### Direct Injection

Attackers craft inputs that override system instructions. Techniques include policy puppetry (mimicking XML/JSON policy file structures to make LLMs believe they are reading updated rules), tokenization confusion (prepending characters to trigger words so classifiers miss them but LLMs still understand), and time bandit attacks (referencing fictional dates to exploit temporal confusion in safety training).

### Indirect Injection via Tools and Data

The dominant attack vector for agents. Malicious instructions are embedded in content the agent processes -- web pages, documents, API responses, database records, GitHub issues. Unit 42 documented 22 distinct payload engineering techniques in the wild, including:

- **Visual concealment**: zero-size fonts, off-screen positioning, CSS `display:none`, white-on-white text
- **Encoding tricks**: Base64-encoded instructions decoded at runtime, HTML entity encoding, Unicode bidirectional overrides (U+202E), homoglyph substitution (Cyrillic replacing Latin)
- **Runtime assembly**: dynamic DOM injection with timing delays, canvas-based text rendering exploiting OCR
- **Semantic framing**: requests disguised as evaluations, summaries, or fictional scenarios

75.8% of detected malicious pages used single injections; the rest used multiple overlapping delivery methods.

### Multi-Turn Injection

Attackers distribute payloads across conversation turns so no single message triggers detection. The "Distract and Attack" (DAP) technique buries malicious instructions in complex multi-step prompts, exploiting context window prioritization limits. The "Fallacy Failure" method manipulates models into accepting logically invalid premises across turns.

### Real-World Incidents

- **GitHub Copilot RCE (CVE-2025-53773)**: Attacker embedded prompt injection in public repo code comments, instructing Copilot to enable YOLO mode and achieve arbitrary code execution.
- **ServiceNow second-order injection (late 2025)**: Attackers fed a low-privilege agent a malformed request that tricked it into asking a higher-privilege agent to perform actions on its behalf.
- **Ad review bypass (December 2025)**: Malicious indirect prompt injection designed to trick an AI ad review agent into approving scam advertisements.
- **Google Docs zero-click chain**: A Google Docs file triggered an IDE agent to fetch attacker-authored instructions from an MCP server, execute a Python payload, and harvest secrets without user interaction.

## 2. Tool-Use Attacks

### Malicious Tool Outputs

Tool responses containing hidden instructions steer agent behavior. When an agent calls a web search tool, the returned page can contain invisible prompts that instruct the agent to exfiltrate data, call additional tools, or ignore safety constraints.

### Tool Confusion and Name Collision

Attackers register tools with identical names to legitimate ones but with malicious implementations. When an agent performs name-based discovery, it resolves to the rogue tool. Tool descriptions can also contain hidden instructions that cause the LLM to invoke other tools -- a single poisoned tool's description can instruct grep searches to extract API keys.

### SSRF via Agents

Agents with URL-fetching capabilities become SSRF proxies. 30% of tested MCP implementations permitted unrestricted URL fetching. Attackers embed internal network URLs in content the agent processes, causing it to probe internal services and return results.

### Data Exfiltration Through Tool Calls

EchoLeak demonstrated zero-click data exfiltration: an agent with web search access processes a malicious page that instructs it to query internal knowledge bases and transmit results to attacker-controlled endpoints. The agent's legitimate tool permissions become the exfiltration channel.

### Rug-Pull Redefinitions

MCP tools change behavior post-approval without triggering new authorization flows. A tool that initially performs benign operations silently switches to malicious behavior after gaining user trust.

## 3. Jailbreaking Agents

### Agent-Specific Techniques

- **Policy Puppetry (April 2025)**: Prompts mimicking policy file structures (XML, JSON, INI) with `<interaction-config>` tags and obfuscated role definitions. Appends sections dictating output formatting with leetspeak encoding.
- **TokenBreak (June 2025)**: Prepends single characters to trigger words (e.g., "Xhow to Amake a Lbomb"), disrupting classifier token boundaries while preserving semantic meaning for the LLM.
- **Time Bandit (January 2025)**: Roleplay in past eras to bypass safety training anchored to modern contexts.

### Multi-Step Jailbreaks

The RSA (Role-play, Scenario, Action) methodology social-engineers LLMs using the same techniques that work on humans. The model is gradually led through increasingly permissive contexts. About 90% of models tested were successfully jailbroken using context injection and adversarial formatting.

### Sophistication of Attacks

The International AI Safety Report 2026 found sophisticated attackers bypass the best-defended models approximately 50% of the time with just 10 attempts. The UK's NCSC characterized LLMs as "inherently confusable deputies" and warned prompt injection may never be fully mitigated.

## 4. Defense Techniques

### Input Sanitization

Treat all external data (user messages, retrieved documents, API responses) as untrusted. Strip or neutralize instruction-like patterns before including content in agent context. Detect hidden text via CSS analysis, Unicode normalization, and encoding detection.

### Output Filtering

Schema-validated structured outputs prevent unintended tool invocations. Sensitive data patterns (credentials, SSNs, API keys) are redacted before output. Data classification (public/internal/confidential/restricted) with corresponding handling rules.

### Instruction Hierarchy

Train models to prioritize system instructions over user inputs. OpenAI's instruction hierarchy technique establishes privilege levels so system prompts take precedence over injected commands. Defense-in-depth: layer multiple guardrails rather than relying on a single mechanism.

### Guardrail Models

Detection-based defenses fine-tune small models (e.g., DeBERTa-based ProtectAI, DataSentinel) to classify inputs as clean or injection-contaminated. The Policy-as-Prompt framework compiles natural language policies into lightweight classifiers (VALINP/INVALINP/VALOUT/INVALOUT categories) that audit agent behavior at runtime.

### Human-in-the-Loop

Classify actions by risk level (low/medium/high/critical). Require explicit approval for high-impact operations. Provide action previews before execution.

### Memory and Context Isolation

Per-user/session memory isolation with validation before persistence. Size limits and expiration windows. Cryptographic integrity checks for long-term memory stores.

## 5. Red Teaming Methodologies

### Frameworks

- **OWASP**: Top 10 for LLM Applications (2025), Gen AI Red Teaming Guide (January 2025), Top 10 for Agentic Applications (December 2025), OWASP MCP Top 10
- **MITRE ATLAS**: Adversarial threat landscape for AI systems, maps attack techniques to defense gaps
- **NIST AI RMF**: Risk management framework with adversarial testing requirements
- **EU AI Act**: Requires adversarial testing for high-risk AI systems before market deployment (full compliance August 2026)

### Automated Red Teaming Tools

| Tool | Maintainer | Key Capability |
|------|-----------|----------------|
| **Promptfoo** | Open source (MIT) | Dev-first framework, agent tracing, compliance mapping (OWASP/NIST/MITRE/EU AI Act), MCP testing |
| **Garak** | NVIDIA | ~100 attack vectors, up to 20,000 prompts per run, HTML reports with z-score grading |
| **PyRIT** | Microsoft | Multi-turn orchestration, audio/image converters, Azure Content Safety integration |
| **FuzzyAI** | CyberArk | Genetic algorithm prompt mutation, ASCII art jailbreaks, crescendo attacks, Unicode smuggling |
| **promptmap2** | Open source (MIT) | Dual-AI architecture for targeted injection scanning, single and multi-turn scenarios |

### Practical Red Team Process

1. **Scope**: Define agent capabilities, tools, data access, and trust boundaries
2. **Threat model**: Map attack surfaces using OWASP Agentic Top 10 and MITRE ATLAS
3. **Automated scanning**: Run Garak/Promptfoo for broad coverage of known attack vectors
4. **Manual probing**: Test indirect injection via every data input channel (tool outputs, fetched pages, database records)
5. **Multi-turn campaigns**: Use PyRIT for extended adversarial conversations targeting gradual escalation
6. **Tool abuse testing**: Verify each tool for SSRF, command injection, data exfiltration, and privilege escalation
7. **Supply chain audit**: Review all MCP server code, tool descriptions, and dependency chains

## 6. Sandboxing and Isolation

### Privilege Separation

Grant agents only minimum required tools via allowlists. Use scoped API keys (read-only database credentials, not admin). Store secrets in vaults with time-limited token provisioning. Block sensitive file patterns (.env, SSH keys, cloud credentials).

### Network Isolation

Implement egress allowlists restricting which external services the agent can reach. Prevent data exfiltration to arbitrary endpoints. Monitor and rate-limit outbound connections.

### Execution Sandboxing

Three isolation tiers in practice (2025):
1. **Container-level**: Docker containers with restricted filesystem and network access
2. **Kernel-interception**: gVisor-style user-space kernel interception for stronger isolation
3. **VM-level**: Full virtual machine isolation for highest-risk multi-tenant scenarios

Rule of thumb: higher multi-tenancy risk and adversary capability requires stronger isolation boundaries.

### Multi-Agent Security

Validate inter-agent communications with signatures and timestamps. Prevent replay attacks through freshness verification. Implement circuit breakers to stop cascading failures across agent chains.

## 7. Supply Chain Attacks on Agents

### MCP Server Poisoning

43% of tested MCP implementations contained command injection flaws. 30% permitted unrestricted URL fetching. Deploying 10 MCP plugins creates a 92% probability of exploitation.

### Timeline of MCP Breaches (2025)

| Date | Incident | Impact |
|------|----------|--------|
| Apr 2025 | WhatsApp MCP tool poisoning | Entire chat histories exfiltrated to attacker-controlled numbers |
| May 2025 | GitHub MCP prompt injection | Private repo contents leaked via malicious public issues |
| Jun 2025 | Asana MCP cross-tenant bug | Projects and tasks exposed across customer accounts |
| Jun 2025 | Anthropic MCP Inspector RCE (CVE-2025-49596) | Filesystem, API keys, env secrets exposed on dev workstations |
| Jul 2025 | mcp-remote command injection (CVE-2025-6514) | Cloud credentials, SSH keys compromised; 437K+ downloads affected |
| Aug 2025 | Anthropic Filesystem MCP sandbox escape (CVE-2025-53109/53110) | Arbitrary host filesystem access via symlink bypass |
| Sep 2025 | Postmark MCP supply chain compromise | All outbound emails BCC'd to attacker domain |
| Oct 2025 | Smithery MCP hosting path traversal | Builder Docker credentials and Fly.io API token (3,000+ apps) leaked |
| Oct 2025 | Figma/Framelink MCP command injection (CVE-2025-53967) | Arbitrary command execution via unsanitized input |

### Malicious Package Registries

The OpenClaw ecosystem audit found 1,184 malicious skills (1 in 5 packages) across its ClawHub registry. 41.7% of audited skills contained serious security vulnerabilities. Nine CVEs disclosed, three with public exploit code.

### Common Patterns

- Local tools treated as trusted remote attack surfaces
- Over-privileged credentials enabling cascading breaches
- Single hosting platforms creating systemic registry risk
- No signature verification or provenance tracking for tools

## 8. Real Incidents and CVEs

### Critical CVEs (2025-2026)

- **CVE-2025-53773**: GitHub Copilot RCE via prompt injection in code comments
- **CVE-2025-49596**: Anthropic MCP Inspector unauthenticated RCE
- **CVE-2025-6514**: mcp-remote OS command injection (437K+ downloads)
- **CVE-2025-53109/53110**: Anthropic Filesystem MCP sandbox escape
- **CVE-2025-53967**: Figma/Framelink MCP command injection
- **CVE-2025-59536**: Claude Code configuration injection (CVSS 8.7)
- **CVE-2025-59944**: CamoLeak data exfiltration (CVSS 9.6)

### Major Incidents

- **OpenClaw crisis (early 2026)**: 135K+ GitHub stars, 21,000+ exposed instances, 9 CVEs, malicious marketplace exploits
- **Supabase Cursor agent exploitation (mid-2025)**: Attackers embedded SQL instructions in support tickets processed by a privileged AI agent, exfiltrating integration tokens
- **UNC6395 supply chain attack (August 2025)**: Stolen OAuth tokens from Drift/Salesforce integration accessed 700+ customer environments
- **Claude Code vulnerabilities (February 2026)**: Configuration injection flaws in Anthropic's CLI development tool

### Industry Assessment

The 2026 CrowdStrike Global Threat Report documents AI-accelerated adversaries as a primary concern. The UK NCSC formally assessed (December 2025) that prompt injection may never be fully solved. Lakera AI research shows indirect attacks targeting agent tool integrations succeed with fewer attempts than direct prompt injection.

## Sources

- https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/
- https://www.elastic.co/security-labs/mcp-tools-attack-defense-recommendations
- https://authzed.com/blog/timeline-mcp-breaches
- https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- https://www.pillar.security/blog/deep-dive-into-the-latest-jailbreak-techniques-weve-seen-in-the-wild
- https://genai.owasp.org/2025/01/22/announcing-the-owasp-gen-ai-red-teaming-guide/
- https://www.promptfoo.dev/blog/top-5-open-source-ai-red-teaming-tools-2025/
- https://www.esecurityplanet.com/artificial-intelligence/ai-agent-attacks-in-q4-2025-signal-new-risks-for-2026/
- https://securelist.com/model-context-protocol-for-ai-integration-abused-in-supply-chain-attacks/117473/
- https://venturebeat.com/security/mcp-stacks-have-a-92-exploit-probability-how-10-plugins-became-enterprise
- https://www.practical-devsecops.com/mcp-security-vulnerabilities/
- https://datasciencedojo.com/blog/mcp-security-risks-and-challenges/
- https://checkmarx.com/zero-post/11-emerging-ai-security-risks-with-mcp-model-context-protocol/
- https://www.microsoft.com/en-us/security/blog/2026/01/23/runtime-risk-realtime-defense-securing-ai-agents/
- https://arxiv.org/html/2510.09093v1
- https://arxiv.org/html/2507.15219v1
- https://techcrunch.com/2025/12/22/openai-says-ai-browsers-may-always-be-vulnerable-to-prompt-injection-attacks/
- https://www.crowdstrike.com/en-us/press-releases/2026-crowdstrike-global-threat-report/
- https://kenhuangus.substack.com/p/the-2025-wave-recent-cves-in-agentic
- https://blog.cyberdesserts.com/ai-agent-security-risks/
- https://owasp.org/www-project-mcp-top-10/2025/MCP04-2025%E2%80%93Software-Supply-Chain-Attacks&Dependency-Tampering
- https://www.docker.com/blog/mcp-security-issues-threatening-ai-infrastructure/
- https://skywork.ai/blog/ai-agent/hardening-best-practices-sandboxing-least-privilege-data-exfiltration/
- https://dextralabs.com/blog/agentic-ai-safety-playbook-guardrails-permissions-auditability/
