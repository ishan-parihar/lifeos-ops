//! ID Resolver — fuzzy name → Notion ID resolution
//!
//! Mirrors the TypeScript src/utils/id-resolver.ts

use crate::config::{LifeOSConfig, get_db};
use crate::notion::client::NotionClient;

/// Result of a resolution attempt
pub struct ResolutionResult {
    pub id: Option<String>,
    pub matches: Option<Vec<String>>,
    pub error: Option<String>,
}

/// Resolve a target name to a Notion ID using fuzzy matching
pub async fn resolve_target_id(
    notion: &NotionClient,
    config: &LifeOSConfig,
    db_key: &str,
    target_name: Option<&str>,
    target_id: Option<&str>,
) -> ResolutionResult {
    if let Some(id) = target_id {
        return ResolutionResult {
            id: Some(id.to_string()),
            matches: None,
            error: None,
        };
    }

    let name = match target_name {
        Some(n) => n,
        None => return ResolutionResult { id: None, matches: None, error: None },
    };

    let db = match get_db(config, db_key) {
        Some(d) => d,
        None => return ResolutionResult {
            id: None,
            matches: None,
            error: Some(format!("Unknown database: {}", db_key)),
        },
    };

    // Query all pages in the database
    let result = match notion.query_data_source(
        &db.data_source_id,
        &serde_json::json!({ "page_size": 100 }),
    ).await {
        Ok(r) => r,
        Err(e) => return ResolutionResult {
            id: None,
            matches: None,
            error: Some(e.to_string()),
        },
    };

    // Match titles using fuzzy matching
    let titles: Vec<(String, String)> = result.results.iter()
        .map(|page| {
            let title = crate::transform::extract_title(page);
            (title, page.id.clone())
        })
        .collect();

    find_best_match(&titles, name)
}

/// Find the best fuzzy match for a name among a list of titles
fn find_best_match(titles: &[(String, String)], query: &str) -> ResolutionResult {
    let lower_query = query.to_lowercase();
    let mut scored: Vec<(f64, &str, &str)> = titles.iter()
        .map(|(title, id)| {
            let lower = title.to_lowercase();
            let score = if lower == lower_query {
                1.0
            } else if lower.contains(&lower_query) || lower_query.contains(&lower) {
                0.85
            } else {
                // Levenshtein similarity
                let dist = strsim::normalized_levenshtein(&lower, &lower_query);
                dist
            };
            (score, title.as_str(), id.as_str())
        })
        .filter(|(score, _, _)| *score > 0.4)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    if scored.is_empty() {
        return ResolutionResult { id: None, matches: None, error: None };
    }

    // Exact or near-exact match
    if scored[0].0 >= 0.9 {
        return ResolutionResult {
            id: Some(scored[0].2.to_string()),
            matches: None,
            error: None,
        };
    }

    // Multiple close matches — return suggestions
    if scored.len() > 1 && scored[0].0 - scored[1].0 < 0.2 {
        return ResolutionResult {
            id: None,
            matches: Some(scored.iter().take(5).map(|(s, t, id)| {
                format!("{} ({}) — match: {:.0}%", t, id, s * 100.0)
            }).collect()),
            error: None,
        };
    }

    // Single best match
    ResolutionResult {
        id: Some(scored[0].2.to_string()),
        matches: None,
        error: None,
    }
}
