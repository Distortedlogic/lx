# Subagent Fine-Tuning

Pipeline from runtime data collection through training to specialist deployment.

## Target Goal

Fine-tune specialist models for high-frequency subagent roles (audit-verifier, classifier, grader, db-query). Collect traces via langfuse during normal operation. When ≥5K scored traces accumulate, harvest them, enhance low-scored traces with a teacher model, format as ChatML, and train QLoRA adapters on Qwen2.5-Coder-32B. Deploy via vLLM with a router that decides specialist vs frontier model per request.

## Scenarios

### Scenario 1: Code Auditor Specialist

Train a specialist that flags CLAUDE.md code style violations. Teacher distillation: send 500 code snippets to frontier model → 400 violations + 100 compliant. Codebase walk: extract .rs files, teacher generates Q&A. Git history: parse 200 commits for rule-fix patterns. Total: 8.5K examples. Train 3 epochs, ~4-5 hours.

**Success:** Precision ≥0.95, recall ≥0.85 on hold-out set. Specialist catches 85%+ of seeded violations with <5% false positive rate.

**Edge case:** Specialist overfits on training patterns — great on eval, fails on new repos. Mitigation: synthetic variation in training data + early stopping.

### Scenario 2: Teacher Enhancement for Low-Scored Traces

Harvest 6K traces for the classifier agent. 2K scored ≥0.7 (good), 4K scored <0.5 (bad). Teacher model receives each low-scored trace + 3 high-scored examples and generates an ideal output. Enhanced dataset: 2K good originals + 4K teacher-improved = 6K total.

**Success:** After training on enhanced data, classifier accuracy improves 15-20% vs training on good traces alone.

### Scenario 3: Specialist Routing Decision

Request arrives: "classify this issue." Router checks: classifier specialist available, confidence ≥ threshold → route to specialist (low latency, low cost). If specialist returns low-confidence result → escalate to frontier model.

**Success:** 70%+ of requests handled by specialist at 10x lower latency. Frontier handles the 30% that need full reasoning.

### Scenario 4: Retraining After Rule Change

CLAUDE.md rules change (new rules added). Auditor specialist quality degrades — false negatives on new rules. Generate new training data for changed rules via teacher. Merge with old dataset. Retrain from base model (not fine-tuning the fine-tuned model). A/B test old vs new adapter in vLLM.

**Success:** New specialist scores ≥0.95 on new rules within 1 week. Full cycle: ~6 hours (data gen + training + eval).

### Scenario 5: Continuous Feedback Loop

Deployed specialist handles live traffic. Each response traced in langfuse. Outcomes scored (did the audit finding hold up? did the classification match human correction?). After 5K new scored traces accumulate, next training cycle auto-triggers.

**Success:** Specialist quality improves monotonically across retraining cycles. Each cycle is smaller (incremental data, not full retrain).
