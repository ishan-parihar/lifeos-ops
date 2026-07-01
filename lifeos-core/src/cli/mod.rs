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

    /// Show database schemas with property types and relation targets
    Schema {
        /// Optional reservoir to filter (matrix, potentiator, significator, greatway, nexus)
        #[arg(short, long)]
        database: Option<String>,
    },

    /// Query a database with filters, sorts, and presets
    Query {
        /// Database key (tasks, projects, activity_log, etc.)
        database: String,
        /// Property to filter on
        #[arg(short, long)]
        filter_property: Option<String>,
        /// Filter value
        #[arg(short, long)]
        filter_value: Option<String>,
        /// Filter type (select, status, rich_text, date, etc.)
        #[arg(long)]
        filter_type: Option<String>,
        /// Sort property
        #[arg(short, long)]
        sort_property: Option<String>,
        /// Sort direction (ascending/descending)
        #[arg(short, long)]
        sort_direction: Option<String>,
        /// Max results (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: u32,
        /// Preset: active, this_week, this_month, needs_review
        #[arg(short, long)]
        preset: Option<String>,
        /// Query a reservoir + all its satellites
        #[arg(short, long)]
        reservoir: Option<String>,
        /// Query all reservoirs in a cycle (lesser/greater)
        #[arg(short, long)]
        cycle: Option<String>,
    },

    /// Create, update, or delete an entry
    Mutate {
        /// Operation: create, update, delete
        #[arg(short, long)]
        operation: String,
        /// Database key
        #[arg(short, long)]
        database: String,
        /// Page ID (for update/delete)
        #[arg(short, long)]
        page_id: Option<String>,
        /// Properties as JSON string (for create/update)
        #[arg(short, long)]
        properties: Option<String>,
        /// Dry run — show what would happen
        #[arg(long)]
        dry_run: bool,
    },

    /// Get a role or cycle intelligence briefing
    Intelligence {
        /// Briefing mode: role, module, lesser_cycle, greater_cycle, nexus, drive_balance, reservoir_health
        #[arg(short, long)]
        mode: String,
        /// Role key (CEO, COO, CMO, CRO, CFO, CHO)
        #[arg(short, long)]
        role: Option<String>,
        /// Module key (productivity, health, strategic, financial, content, journaling)
        #[arg(short, long)]
        module: Option<String>,
        /// Date range: today, this_week, this_month, this_quarter
        #[arg(short, long)]
        range: Option<String>,
    },

    /// Run data science analysis (temporal, trajectories, correlations)
    DataScience {
        /// Analysis type: temporal, trajectories, weekday_profile, correlations, patterns
        #[arg(short, long)]
        analysis_type: String,
        /// Primary database key
        #[arg(short, long)]
        database: String,
        /// Secondary database (for correlations)
        #[arg(short, long)]
        database_b: Option<String>,
        /// Days to look back (default: 30)
        #[arg(short, long)]
        days_back: Option<i64>,
        /// Property to analyze
        #[arg(short, long)]
        property: Option<String>,
        /// Metric property for numerical analysis
        #[arg(short, long)]
        metric_property: Option<String>,
    },

    /// Run daily/weekly/monthly/quarterly review pipeline
    Review {
        /// Review type: daily, weekly, monthly, quarterly
        #[arg(short, long)]
        review_type: String,
    },

    /// Cross-DB strategic analysis: OKRs, projects, campaigns
    Strategic {
        /// Analysis type: overview, alignment, project_health, okr_progress, campaign_metrics
        #[arg(short, long)]
        analysis_type: String,
        /// Project database key
        #[arg(short, long)]
        project_database: Option<String>,
        /// OKR database key
        #[arg(short, long)]
        okr_database: Option<String>,
        /// Campaign database key
        #[arg(short, long)]
        campaign_database: Option<String>,
    },

    /// Trace currency flow across the holonic spiral
    EnergyFlow {
        /// Scope: lesser_cycle, greater_cycle, full_spiral, or specific reservoir
        #[arg(short, long)]
        scope: String,
        /// Currency to trace: Catalyst, Experience, Transformation, Choice, all
        #[arg(short, long)]
        currency: Option<String>,
        /// Limit per database (default: 10)
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Assess 4 drives at lesser/greater boundary
    DriveAssessment {
        /// Boundary: lesser, greater, both
        #[arg(short, long, default_value = "both")]
        boundary: String,
        /// Date range
        #[arg(short, long)]
        range: Option<String>,
    },

    /// Holonic health metrics (G_z + P_z)
    HealthMetrics {
        /// Metric: lesser, greater, both
        #[arg(short, long, default_value = "both")]
        metric: String,
        /// Date range
        #[arg(short, long)]
        range: Option<String>,
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
