use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lifeos", version, about = "LifeOS unified CLI + MCP server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the vault directory structure
    Init {
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Pull all pages from Notion to vault
    Pull {
        /// Database keys to pull (comma-separated), or all if omitted
        #[arg(short, long)]
        databases: Option<String>,

        /// Database keys to exclude (comma-separated)
        #[arg(long)]
        exclude: Option<String>,

        /// Incremental pull: only fetch pages edited since last pull
        #[arg(long)]
        incremental: bool,

        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Push changes from vault back to Notion
    Push {
        /// Database keys to push (comma-separated), or all if omitted
        #[arg(short, long)]
        databases: Option<String>,

        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,

        /// Dry run — show what would change without modifying Notion
        #[arg(long)]
        dry_run: bool,
    },

    /// Watch vault for changes and push to Notion in real-time
    Watch {
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,

        /// Debounce delay in milliseconds (default: 2000)
        #[arg(long, default_value = "2000")]
        debounce_ms: u64,
    },

    /// Manage pages: create, edit, diff, merge
    Page {
        #[command(subcommand)]
        action: PageCommand,
    },

    /// Fetch a single entry with all relations resolved to titles
    GetPage {
        /// Notion page ID
        page_id: String,
        /// Optional database key hint
        #[arg(short, long)]
        database: Option<String>,
    },

    /// Batch resolve relation IDs to titled entries
    Expand {
        /// Comma-separated page IDs to expand
        ids: String,
    },

    /// Follow relations N levels deep from any entry
    Trace {
        /// Starting page ID
        page_id: String,
        /// Max depth (default: 2, max: 3)
        #[arg(short, long, default_value = "2")]
        depth: u32,
    },

    /// Walk up hierarchy from entry to root (task→project→QG→AG)
    Ancestors {
        /// Starting page ID
        page_id: String,
        /// Max levels to walk up (default: 5)
        #[arg(short, long, default_value = "5")]
        max_levels: u32,
    },

    /// Run as MCP server (stdio JSON-RPC)
    MCP,

    /// Scan Notion for databases and update config with correct IDs
    Discover {
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum PageCommand {
    /// Create a new page in Notion and pull to vault
    New {
        /// Database key (e.g. tasks, projects)
        db_key: String,
        /// Page title
        title: String,
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Edit a page in $EDITOR with 3-way merge on save
    Edit {
        /// Page UUID (from vault filename or wiki-link)
        page_id: String,
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Show property and body diff between vault and Notion
    Diff {
        /// Page UUID
        page_id: String,
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Resolve merge conflicts and push to Notion
    Merge {
        /// Page UUID
        page_id: String,
        /// Path to lifeos.config.json
        #[arg(short, long)]
        config: Option<String>,
    },
}
