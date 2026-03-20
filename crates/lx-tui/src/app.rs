use std::collections::HashSet;

use lx_dx::event::RuntimeEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Ai,
    Emit,
    Log,
    Shell,
    Messages,
    Agents,
    Progress,
    Errors,
}

impl EventCategory {
    pub fn all() -> HashSet<Self> {
        HashSet::from([
            Self::Ai,
            Self::Emit,
            Self::Log,
            Self::Shell,
            Self::Messages,
            Self::Agents,
            Self::Progress,
            Self::Errors,
        ])
    }
}

pub struct App {
    pub events: Vec<(RuntimeEvent, EventCategory)>,
    pub filters: HashSet<EventCategory>,
    pub scroll: usize,
    pub agent_filter: Option<String>,
    pub agents: Vec<String>,
    pub cumulative_cost: f64,
    pub elapsed_ms: u64,
    pub program_status: Option<Result<String, String>>,
    pub source_path: String,
    pub should_quit: bool,
    pub program_started: bool,
}

const MAX_EVENTS: usize = 10_000;

impl App {
    pub fn new(source_path: String) -> Self {
        Self {
            events: Vec::new(),
            filters: EventCategory::all(),
            scroll: 0,
            agent_filter: None,
            agents: Vec::new(),
            cumulative_cost: 0.0,
            elapsed_ms: 0,
            program_status: None,
            source_path,
            should_quit: false,
            program_started: false,
        }
    }

    pub fn categorize(event: &RuntimeEvent) -> EventCategory {
        match event {
            RuntimeEvent::AiCallStart { .. }
            | RuntimeEvent::AiCallComplete { .. }
            | RuntimeEvent::AiCallError { .. } => EventCategory::Ai,
            RuntimeEvent::Emit { .. } => EventCategory::Emit,
            RuntimeEvent::Log { .. } => EventCategory::Log,
            RuntimeEvent::ShellExec { .. } | RuntimeEvent::ShellResult { .. } => {
                EventCategory::Shell
            }
            RuntimeEvent::MessageSend { .. }
            | RuntimeEvent::MessageAsk { .. }
            | RuntimeEvent::MessageResponse { .. }
            | RuntimeEvent::UserPrompt { .. }
            | RuntimeEvent::UserResponse { .. } => EventCategory::Messages,
            RuntimeEvent::AgentSpawned { .. } | RuntimeEvent::AgentKilled { .. } => {
                EventCategory::Agents
            }
            RuntimeEvent::Progress { .. }
            | RuntimeEvent::ProgramStarted { .. }
            | RuntimeEvent::ProgramFinished { .. }
            | RuntimeEvent::TraceSpanRecorded { .. } => EventCategory::Progress,
            RuntimeEvent::Error { .. } => EventCategory::Errors,
        }
    }

    pub fn push_event(&mut self, event: RuntimeEvent) {
        match &event {
            RuntimeEvent::AiCallComplete {
                cost_usd: Some(cost),
                ..
            } => {
                self.cumulative_cost += cost;
            }
            RuntimeEvent::AgentSpawned { agent_id, .. } => {
                if !self.agents.contains(agent_id) {
                    self.agents.push(agent_id.clone());
                }
            }
            RuntimeEvent::ProgramStarted { .. } => {
                self.program_started = true;
            }
            RuntimeEvent::ProgramFinished {
                result,
                duration_ms,
                ..
            } => {
                self.elapsed_ms = *duration_ms;
                self.program_status = Some(result.clone());
            }
            _ => {}
        }
        let cat = Self::categorize(&event);
        self.events.push((event, cat));
        if self.events.len() > MAX_EVENTS {
            self.events.remove(0);
        }
    }

    pub fn visible_events(&self) -> Vec<&RuntimeEvent> {
        self.events
            .iter()
            .filter(|(event, cat)| {
                if !self.filters.contains(cat) {
                    return false;
                }
                if let Some(ref agent) = self.agent_filter {
                    return event.agent_id().is_some_and(|id| id == agent);
                }
                true
            })
            .map(|(event, _)| event)
            .collect()
    }

    pub fn toggle_filter(&mut self, cat: EventCategory) {
        if self.filters.contains(&cat) {
            self.filters.remove(&cat);
        } else {
            self.filters.insert(cat);
        }
    }

    pub fn reset_filters(&mut self) {
        self.filters = EventCategory::all();
    }

    pub fn cycle_agent(&mut self) {
        if self.agents.is_empty() {
            self.agent_filter = None;
            return;
        }
        match &self.agent_filter {
            None => {
                self.agent_filter = Some(self.agents[0].clone());
            }
            Some(current) => {
                if let Some(idx) = self.agents.iter().position(|a| a == current) {
                    if idx + 1 < self.agents.len() {
                        self.agent_filter = Some(self.agents[idx + 1].clone());
                    } else {
                        self.agent_filter = None;
                    }
                } else {
                    self.agent_filter = None;
                }
            }
        }
    }
}
