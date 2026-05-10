use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::DebounceEventResult;
use notify_debouncer_mini::DebouncedEvent;
use notify_debouncer_mini::notify::RecursiveMode;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::vault;

use super::push_database;

pub async fn watch_vault(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    debounce_ms: u64,
) -> Result<(), String> {
    let vault_dir = vault_dir.to_path_buf();
    if !vault_dir.exists() {
        return Err(format!(
            "Vault directory not found: {}",
            vault_dir.display()
        ));
    }

    let (tx, mut rx) =
        tokio::sync::mpsc::unbounded_channel::<DebounceEventResult>();

    let watch_dir = vault_dir.clone();
    tokio::task::spawn_blocking(move || {
        let (event_tx, event_rx) = mpsc::channel::<DebounceEventResult>();

        let mut debouncer = match new_debouncer(Duration::from_millis(debounce_ms), event_tx) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Failed to create file watcher: {e}");
                return;
            }
        };

        if let Err(e) = debouncer
            .watcher()
            .watch(&watch_dir, RecursiveMode::Recursive)
        {
            tracing::error!("Failed to watch directory: {e}");
            return;
        }

        tracing::info!("Watching {} for changes...", watch_dir.display());

        while let Ok(result) = event_rx.recv() {
            if tx.send(result).is_err() {
                break;
            }
        }
    });

    while let Some(result) = rx.recv().await {
        match result {
            Ok(events) => {
                let affected_keys = collect_affected_db_keys(&events, &vault_dir);
                for db_key in affected_keys {
                    tracing::info!(
                        "Change detected in database '{}', pushing...",
                        db_key
                    );

                    let index = match vault::read_index(&vault_dir) {
                        Ok(idx) => idx,
                        Err(e) => {
                            tracing::warn!("Read index: {e}");
                            continue;
                        }
                    };

                    if let Err(e) = push_database(
                        notion, config, &db_key, &vault_dir, &index, false,
                    )
                    .await
                    {
                        tracing::error!("Push '{}' failed: {e}", db_key);
                    } else {
                        tracing::info!("Push '{}' completed", db_key);
                    }
                }
            }
            Err(err) => {
                tracing::warn!("Watch error: {err}");
            }
        }
    }

    Ok(())
}

fn collect_affected_db_keys(
    events: &[DebouncedEvent],
    vault_dir: &Path,
) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();

    for event in events {
        if let Ok(rel) = event.path.strip_prefix(vault_dir) {
            let components: Vec<_> = rel.components().collect();
            if components.len() >= 2
                && event.path.extension().map_or(false, |e| e == "md")
            {
                if let Some(first) = components.first() {
                    let key = first.as_os_str().to_string_lossy().to_string();
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                }
            }
        }
    }

    keys.sort();
    keys
}
