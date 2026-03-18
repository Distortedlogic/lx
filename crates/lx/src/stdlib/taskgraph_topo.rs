use indexmap::IndexMap;

use crate::error::LxError;
use crate::span::Span;

use super::TaskNode;

pub(super) fn topo_sort(
    nodes: &IndexMap<String, TaskNode>,
    span: Span,
) -> Result<Vec<String>, LxError> {
    validate_deps(nodes, span)?;

    let mut in_deg: IndexMap<&str, usize> = IndexMap::new();
    for (id, node) in nodes {
        in_deg.insert(id.as_str(), node.depends.len());
    }

    let mut queue: Vec<&str> = in_deg
        .iter()
        .filter(|(_, c)| **c == 0)
        .map(|(id, _)| *id)
        .collect();
    queue.sort();

    let mut result = Vec::new();
    while let Some(id) = queue.pop() {
        result.push(id.to_string());
        for (nid, node) in nodes.iter() {
            if node.depends.iter().any(|d| d == id)
                && let Some(c) = in_deg.get_mut(nid.as_str())
            {
                *c -= 1;
                if *c == 0 {
                    queue.push(nid.as_str());
                    queue.sort();
                }
            }
        }
    }

    if result.len() != nodes.len() {
        return Err(LxError::runtime("taskgraph: cycle detected", span));
    }
    Ok(result)
}

pub(super) fn topo_waves(
    nodes: &IndexMap<String, TaskNode>,
    span: Span,
) -> Result<Vec<Vec<String>>, LxError> {
    validate_deps(nodes, span)?;

    let mut in_deg: IndexMap<&str, usize> = IndexMap::new();
    for (id, node) in nodes {
        in_deg.insert(id.as_str(), node.depends.len());
    }

    let mut waves = Vec::new();
    let mut processed = 0;
    loop {
        let mut wave: Vec<String> = in_deg
            .iter()
            .filter(|(_, c)| **c == 0)
            .map(|(id, _)| (*id).to_string())
            .collect();
        if wave.is_empty() {
            break;
        }
        wave.sort();
        for id in &wave {
            in_deg.shift_remove(id.as_str());
            for (nid, node) in nodes.iter() {
                if node.depends.iter().any(|d| d == id)
                    && let Some(c) = in_deg.get_mut(nid.as_str())
                {
                    *c -= 1;
                }
            }
        }
        processed += wave.len();
        waves.push(wave);
    }

    if processed != nodes.len() {
        return Err(LxError::runtime("taskgraph: cycle detected", span));
    }
    Ok(waves)
}

fn validate_deps(nodes: &IndexMap<String, TaskNode>, span: Span) -> Result<(), LxError> {
    for (_, node) in nodes {
        for dep in &node.depends {
            if !nodes.contains_key(dep) {
                return Err(LxError::runtime(
                    format!("taskgraph: unknown dependency '{dep}'"),
                    span,
                ));
            }
        }
    }
    Ok(())
}
