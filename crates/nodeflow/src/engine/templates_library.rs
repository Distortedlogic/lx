use anyhow::{Context, Result};
use lx_graph_editor::model::GraphDocument;
use serde_json::json;

pub fn built_in_templates() -> Vec<TemplateSummary> {
  vec![
    TemplateSummary {
      id: "slack-daily-brief".to_string(),
      name: "Slack Daily Brief".to_string(),
      description: "Cron -> HTTP -> Summarize with Anthropic -> Slack post".to_string(),
      json: SLACK_DAILY_BRIEF.to_string(),
    },
    TemplateSummary {
      id: "github-issue-on-webhook".to_string(),
      name: "Webhook -> GitHub Issue".to_string(),
      description: "Webhook trigger creates a GitHub issue with the request body.".to_string(),
      json: GITHUB_ISSUE_ON_WEBHOOK.to_string(),
    },
    TemplateSummary {
      id: "sqlite-log-writer".to_string(),
      name: "Manual -> SQLite Log Writer".to_string(),
      description: "Manual trigger writes a row to a SQLite table.".to_string(),
      json: SQLITE_LOG_WRITER.to_string(),
    },
  ]
}

#[derive(Clone, Debug)]
pub struct TemplateSummary {
  pub id: String,
  pub name: String,
  pub description: String,
  pub json: String,
}

pub fn materialize_template(template_id: &str, target_flow_id: &str) -> Result<GraphDocument> {
  let template =
    built_in_templates().into_iter().find(|candidate| candidate.id == template_id).with_context(|| format!("template `{template_id}` not found"))?;
  let mut document: GraphDocument = serde_json::from_str(&template.json).context("failed to parse built-in template json")?;
  document.id = target_flow_id.to_string();
  Ok(document)
}

pub fn export_flow_json(document: &GraphDocument) -> Result<String> {
  serde_json::to_string_pretty(document).context("failed to serialize flow document")
}

const SLACK_DAILY_BRIEF: &str = include_str!("../../assets/flows/template-slack-brief.json");
const GITHUB_ISSUE_ON_WEBHOOK: &str = include_str!("../../assets/flows/template-github-webhook.json");
const SQLITE_LOG_WRITER: &str = include_str!("../../assets/flows/template-sqlite-log.json");

pub fn json_sanity() -> serde_json::Value {
  json!({ "ok": true })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn built_in_templates_parse() {
    for template in built_in_templates() {
      let document = materialize_template(&template.id, "target").expect("template parses");
      assert_eq!(document.id, "target");
      assert!(!document.nodes.is_empty());
    }
  }
}
