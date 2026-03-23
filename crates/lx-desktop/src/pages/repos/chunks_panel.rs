use dioxus::prelude::*;

struct Chunk {
    id: &'static str,
    score: f64,
    description: &'static str,
    tags: &'static [&'static str],
}

const CHUNKS: &[Chunk] = &[
    Chunk {
        id: "#CHUNK_0042",
        score: 0.984,
        description: "Extracted logic for recursive vectorization within high-density AST nodes. Prioritizes semantic weighting over syntactic position.",
        tags: &["VECTOR_R", "RECURSIVE"],
    },
    Chunk {
        id: "#CHUNK_0119",
        score: 0.812,
        description: "Global configuration of the transformer pipeline. Includes context window limits and chunk overlap strategies.",
        tags: &["CONFIG"],
    },
    Chunk {
        id: "#CHUNK_0054",
        score: 0.445,
        description: "Redundant boilerplate found in license headers and environment setup scripts.",
        tags: &[],
    },
];

#[component]
pub fn ChunksPanel() -> Element {
    rsx! {
        div { class: "w-72 bg-[var(--surface-container-low)] border-l border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0 overflow-auto",
            div { class: "flex items-center justify-between mb-4",
                span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
                    "EXTRACTED CHUNKS"
                }
                span { class: "text-xs text-[var(--primary)] cursor-pointer uppercase tracking-wider",
                    "SYNC_JB"
                }
            }
            div { class: "flex flex-col gap-3 flex-1",
                for chunk in CHUNKS {
                    {
                        let score_color = if chunk.score > 0.9 {
                            "bg-[var(--success)]"
                        } else if chunk.score > 0.7 {
                            "bg-[var(--warning)]"
                        } else {
                            "bg-[var(--error)]"
                        };
                        let score_text = format!("{:.3}", chunk.score);
                        rsx! {
                            div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-3",
                                div { class: "flex items-center justify-between mb-2",
                                    span { class: "text-xs text-[var(--primary)] font-semibold", "{chunk.id}" }
                                    span { class: "{score_color} text-[var(--on-primary)] text-[10px] px-2 py-0.5 rounded font-semibold",
                                        "{score_text}"
                                    }
                                }
                                p { class: "text-[10px] text-[var(--on-surface-variant)] leading-relaxed mb-2",
                                    "{chunk.description}"
                                }
                                if !chunk.tags.is_empty() {
                                    div { class: "flex gap-1",
                                        for tag in chunk.tags.iter() {
                                            span { class: "bg-[var(--surface-container-high)] text-[10px] text-[var(--outline)] px-2 py-0.5 rounded uppercase tracking-wider",
                                                "{tag}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "mt-4 pt-3 border-t border-[var(--outline-variant)]/15",
                p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
                    "ANALYSIS STATS"
                }
                p { class: "text-xs text-[var(--on-surface-variant)]", "TOTAL_TOKENS: 1,824,451" }
                p { class: "text-xs text-[var(--on-surface-variant)]", "EMBEDDING_LATENCY: 14ms" }
            }
        }
    }
}
