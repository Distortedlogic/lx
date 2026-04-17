use std::collections::HashMap;

use lx_graph_editor::model::GraphDocument;

pub fn topological_node_ids(document: &GraphDocument) -> Vec<String> {
  let mut indegree: HashMap<String, usize> = document.nodes.iter().map(|node| (node.id.clone(), 0usize)).collect();
  let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
  let positions: HashMap<String, (f64, f64)> = document.nodes.iter().map(|node| (node.id.clone(), (node.position.x, node.position.y))).collect();

  for edge in &document.edges {
    if indegree.contains_key(&edge.from.node_id) && indegree.contains_key(&edge.to.node_id) {
      *indegree.entry(edge.to.node_id.clone()).or_insert(0) += 1;
      outgoing.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
  }

  let mut ready: Vec<String> = indegree.iter().filter(|(_, degree)| **degree == 0).map(|(node_id, _)| node_id.clone()).collect();
  sort_node_ids(&mut ready, &positions);

  let mut ordered = Vec::with_capacity(document.nodes.len());
  while let Some(node_id) = ready.first().cloned() {
    ready.remove(0);
    ordered.push(node_id.clone());
    if let Some(targets) = outgoing.get(&node_id) {
      for target in targets {
        if let Some(degree) = indegree.get_mut(target) {
          *degree = degree.saturating_sub(1);
          if *degree == 0 {
            ready.push(target.clone());
          }
        }
      }
      sort_node_ids(&mut ready, &positions);
    }
  }

  if ordered.len() < document.nodes.len() {
    let mut remaining: Vec<_> =
      document.nodes.iter().map(|node| node.id.clone()).filter(|node_id| !ordered.iter().any(|ordered_id| ordered_id == node_id)).collect();
    sort_node_ids(&mut remaining, &positions);
    ordered.extend(remaining);
  }

  ordered
}

fn sort_node_ids(node_ids: &mut [String], positions: &HashMap<String, (f64, f64)>) {
  node_ids.sort_by(|left, right| {
    let left_position = positions.get(left).copied().unwrap_or((0.0, 0.0));
    let right_position = positions.get(right).copied().unwrap_or((0.0, 0.0));
    left_position
      .0
      .partial_cmp(&right_position.0)
      .unwrap_or(std::cmp::Ordering::Equal)
      .then_with(|| left_position.1.partial_cmp(&right_position.1).unwrap_or(std::cmp::Ordering::Equal))
      .then_with(|| left.cmp(right))
  });
}
