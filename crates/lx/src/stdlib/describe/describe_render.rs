use super::describe_visitor::ProgramDescription;

pub(super) fn render_description(desc: &ProgramDescription) -> String {
    let mut out = String::new();

    if !desc.imports.is_empty() {
        out.push_str("Imports:\n  ");
        let paths: Vec<&str> = desc.imports.iter().map(|i| i.path.as_str()).collect();
        out.push_str(&paths.join(", "));
        out.push('\n');
    }

    if !desc.agents.is_empty() {
        out.push_str("\nAgents:\n");
        for a in &desc.agents {
            let kind = if a.declared { "declared" } else { "spawned" };
            let traits_str = if a.traits.is_empty() {
                "none".to_string()
            } else {
                a.traits.join(", ")
            };
            out.push_str(&format!(
                "  - {} ({}, traits: {})",
                a.name, kind, traits_str
            ));
            if !a.methods.is_empty() {
                out.push_str(&format!(", methods: [{}]", a.methods.join(", ")));
            }
            out.push('\n');
        }
    }

    if !desc.messages.is_empty() {
        out.push_str("\nMessage Flow:\n");
        for m in &desc.messages {
            let label = if m.label.is_empty() {
                String::new()
            } else {
                format!(": \"{}\"", m.label)
            };
            out.push_str(&format!(
                "  {} -> {}{} ({})\n",
                m.from, m.to, label, m.style
            ));
        }
    }

    if !desc.control_flow.is_empty() {
        out.push_str("\nControl Flow:\n");
        for c in &desc.control_flow {
            if c.label.is_empty() {
                out.push_str(&format!("  {}\n", c.kind));
            } else {
                out.push_str(&format!("  {}: {}\n", c.kind, c.label));
            }
        }
    }

    if !desc.resources.is_empty() {
        out.push_str("\nResources:\n");
        for r in &desc.resources {
            out.push_str(&format!("  {}: {} ({})\n", r.kind, r.name, r.source));
        }
    }

    if !desc.ai_calls.is_empty() {
        out.push_str("\nAI Calls:\n");
        for a in &desc.ai_calls {
            out.push_str(&format!("  {}\n", a.context));
        }
    }

    if !desc.exports.is_empty() {
        out.push_str("\nExports:\n  ");
        out.push_str(&desc.exports.join(", "));
        out.push('\n');
    }

    out
}
