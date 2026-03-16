#[path = "agents_grader_rubric.rs"]
mod agents_grader_rubric;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert(
        "grade".into(),
        mk("grader.grade", 1, agents_grader_rubric::bi_grade),
    );
    m.insert(
        "quick_grade".into(),
        mk(
            "grader.quick_grade",
            1,
            agents_grader_rubric::bi_quick_grade,
        ),
    );
    m
}

pub(super) struct GradeFields {
    pub work: String,
    pub task: String,
    pub rubric: Vec<RubricCategory>,
    pub previous_grades: Vec<PrevGrade>,
    pub threshold: i64,
}

pub(super) struct RubricCategory {
    pub name: String,
    pub description: String,
    pub weight: i64,
}

pub(super) struct PrevGrade {
    pub name: String,
    pub score: i64,
    pub passed: bool,
}

pub(super) fn extract_fields(args: &[Value], span: Span) -> Result<GradeFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("grader expects Record", span));
    };
    let work = fields
        .get("work")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("grader: missing 'work' (Str)", span))?
        .to_string();
    let task = fields
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("grader: missing 'task' (Str)", span))?
        .to_string();
    let threshold = fields
        .get("threshold")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(70);
    let rubric = fields
        .get("rubric")
        .and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(extract_rubric_cat).collect())
        .unwrap_or_default();
    let previous_grades = fields
        .get("previous_grades")
        .and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(extract_prev_grade).collect())
        .unwrap_or_default();
    Ok(GradeFields {
        work,
        task,
        rubric,
        previous_grades,
        threshold,
    })
}

fn extract_rubric_cat(v: &Value) -> Option<RubricCategory> {
    let Value::Record(r) = v else { return None };
    Some(RubricCategory {
        name: r.get("name").and_then(|v| v.as_str())?.to_string(),
        description: r
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        weight: r
            .get("weight")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .unwrap_or(1),
    })
}

fn extract_prev_grade(v: &Value) -> Option<PrevGrade> {
    let Value::Record(r) = v else { return None };
    Some(PrevGrade {
        name: r.get("name").and_then(|v| v.as_str())?.to_string(),
        score: r
            .get("score")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .unwrap_or(0),
        passed: r.get("passed").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

pub(super) fn categories_to_evaluate<'a>(
    rubric: &'a [RubricCategory],
    prev: &[PrevGrade],
) -> (Vec<&'a RubricCategory>, Vec<(String, i64)>) {
    if prev.is_empty() {
        return (rubric.iter().collect(), Vec::new());
    }
    let mut to_eval = Vec::new();
    let mut kept = Vec::new();
    for cat in rubric {
        let prev_grade = prev.iter().find(|p| p.name == cat.name);
        match prev_grade {
            Some(pg) if pg.passed => kept.push((cat.name.clone(), pg.score)),
            _ => to_eval.push(cat),
        }
    }
    (to_eval, kept)
}

pub(super) fn build_system_prompt(categories: &[&RubricCategory]) -> String {
    let mut p = String::from(
        "You are a grader scoring work against a rubric. \
         For each category, assign a score 0-100 and brief feedback.\n\n\
         Categories to evaluate:\n",
    );
    for (i, cat) in categories.iter().enumerate() {
        p.push_str(&format!("{}. {} — {}\n", i + 1, cat.name, cat.description));
    }
    p.push_str(
        "\nRespond with ONLY a JSON object, no markdown fences:\n\
         {\"categories\": [{\"name\": \"...\", \"score\": 0-100, \"feedback\": \"...\"}]}",
    );
    p
}

pub(super) fn build_user_prompt(fields: &GradeFields) -> String {
    format!("TASK: {}\n\nWORK TO GRADE:\n{}", fields.task, fields.work)
}
