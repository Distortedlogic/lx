# Post-Hoc Review

Post-task reflection — review transcripts and extract compounding learnings.

## Target Goal

After a task completes, a separate review agent reads the transcript and distills learnings into persistent markdown files: experience.md (patterns that worked + mistakes to avoid) and environment.md (facts about the system). The review runs in a separate context with zero overhead on the primary task. Learnings compound: run 1 captures basics, run 10 forms patterns, run 100 has a rich knowledge base.

## Scenarios

### Scenario 1: Successful Task — Extract Patterns

Agent completed a migration task. Transcript shows: used `postgresql:query` to check schema, wrote migration, ran `cargo build` to verify. Review extracts pattern: "check schema before writing migration" (used successfully 3 times in transcript).

**Success:** experience.md gains a new entry: "check schema before writing migration — confirmed 3x." Entry is actionable for future migration tasks.

### Scenario 2: Failed Task — Extract Mistakes

Agent tried to fix a bug but got stuck. Transcript shows: read file, made wrong edit, tests failed, reverted, read same file again, made similar wrong edit. Review extracts mistake: "tried to fix auth module without reading the token validation flow first."

**Success:** experience.md gains a mistake entry with lesson: "read the full auth flow before editing token validation."

### Scenario 3: Environment Discovery

During a task, agent ran `$^uname -a` and `$^cat /etc/os-release`. Review extracts: "running Fedora 43, kernel 6.19.7." Also ran `$^pwd` → "/home/entropybender/repos/lx".

**Success:** environment.md gains system facts. Future sessions don't need to re-discover the OS or working directory.

### Scenario 4: Compounding Over Time

After 50 review sessions, experience.md has 30 patterns and 15 mistakes. New session reads this at startup. Agent avoids the 15 known mistakes and follows the 30 known-good patterns without re-learning them.

**Success:** Task completion time decreases. Error rate decreases. The agent's "experience" file is a real asset.

### Scenario 5: No Learnings

Agent completed a trivial task (echo "hello"). Transcript has 2 tool calls. Nothing worth extracting.

**Success:** Review runs quickly and adds nothing. experience.md unchanged. No noise entries created.

### Scenario 6: Langfuse Capture for Training

Review identifies a high-quality transcript (task completed efficiently, no mistakes). Creates a langfuse dataset item for fine-tuning data collection.

**Success:** Transcript becomes part of the training pipeline. Score attached reflects quality.
