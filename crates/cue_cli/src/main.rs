//! CueDeck CLI - Command-line interface for CueDeck workspace management
//!
//! Usage: cue <command> [options]

use clap::{Parser, Subcommand};
use cue_common::EXIT_ERROR;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "cue",
    version = "0.1.0",
    about = "CueDeck workspace management"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose/debug logging
    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new CueDeck workspace
    Init,

    /// Generate SCENE.md from active cards
    Scene {
        /// Output to stdout instead of clipboard
        #[arg(short = 'd', long)]
        dry_run: bool,

        /// Override config token limit
        #[arg(long, alias = "limit")]
        token_limit: Option<usize>,
    },

    /// Launch interactive TUI file finder
    Open {
        /// Optional initial search query
        query: Option<String>,

        /// Search mode: keyword, semantic, or hybrid (default)
        #[arg(long, default_value = "hybrid")]
        mode: String,

        /// Use semantic search (deprecated, use --mode=semantic)
        #[arg(long)]
        semantic: bool,

        /// Filter by tags (comma-separated, e.g., "auth,api")
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Filter by priority (e.g., "high", "medium", "low")
        #[arg(long)]
        priority: Option<String>,

        /// Filter by assignee (e.g., "@tctri")
        #[arg(long)]
        assignee: Option<String>,
    },

    /// Watch for file changes and auto-regenerate scene
    Watch,

    /// Run diagnostics on workspace health
    Doctor {
        /// Attempt automatic fixes
        #[arg(long)]
        repair: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Normalize tags to lowercase during repair
        #[arg(long)]
        normalize_tags: bool,
    },

    /// Manage implementation tasks
    Card {
        #[command(subcommand)]
        action: CardAction,
    },

    /// List all cards (alias for 'card list')
    List {
        /// Filter by status (active, done, archived, all)
        #[arg(long, default_value = "active")]
        status: String,

        /// Filter by tags (comma-separated, OR logic)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Filter by priority (critical, high, medium, low)
        #[arg(long)]
        priority: Option<String>,

        /// Filter by assignee (e.g., @username)
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by created date (YYYY, YYYY-MM, YYYY-MM-DD, >2w, <7d)
        #[arg(long)]
        created: Option<String>,

        /// Filter by updated date (YYYY, YYYY-MM, YYYY-MM-DD, >2w, <7d)
        #[arg(long)]
        updated: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Hard reset of cache
    Clean {
        /// Also clear log files
        #[arg(long)]
        logs: bool,
    },

    /// Manage log files
    Logs {
        #[command(subcommand)]
        action: LogAction,
    },

    /// Self-update CueDeck to latest version
    Upgrade,

    /// Start MCP server (JSON-RPC over stdio)
    Mcp,

    /// Search documents (non-interactive, for tool integration)
    Search {
        /// Search query
        query: String,

        /// Search mode: keyword, semantic, or hybrid (default)
        #[arg(long, default_value = "hybrid")]
        mode: String,

        /// Filter by tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Filter by priority
        #[arg(long)]
        priority: Option<String>,

        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Visualize dependency graph
    Graph {
        /// Output format
        #[arg(long, default_value = "ascii")]
        format: String,

        /// Write to file instead of stdout
        #[arg(long)]
        output: Option<String>,

        /// Show graph statistics
        #[arg(long)]
        stats: bool,

        /// Output as JSON (for tool integration)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum CardAction {
    /// Create a new card
    New {
        title: String,
    },

    /// Create a new card with metadata
    Create {
        /// Task title
        title: String,

        /// Tags (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Priority: low, medium, high, critical
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// Assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Task IDs this depends on (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        depends_on: Option<Vec<String>>,
    },

    /// Show task dependencies
    Deps {
        /// Task ID to query
        id: String,

        /// Show dependents instead of dependencies
        #[arg(short, long)]
        reverse: bool,
    },

    /// Validate task dependency graph
    Validate {
        /// Validate specific task only
        id: Option<String>,
    },

    /// Visualize task dependency graph
    Graph {
        /// Output format: dot, mermaid, json
        #[arg(short, long, default_value = "mermaid")]
        format: String,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// List all cards
    List {
        #[arg(long, default_value = "active")]
        status: String,
    },

    /// Open card in $EDITOR
    Edit { id: String },

    /// Move card to archived status
    Archive { id: String },
}

#[derive(Subcommand)]
enum LogAction {
    /// Rotate and compress old logs
    Archive,

    /// Remove all log files
    Clear,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize structured logging via centralized telemetry module
    cue_common::telemetry::init_tracing(cli.verbose, false);
    tracing::info!("CueDeck CLI started");

    let result = match cli.command {
        Commands::Init => cmd_init().await,
        Commands::Scene {
            dry_run,
            token_limit,
        } => cmd_scene(dry_run, token_limit).await,
        Commands::Open {
            query,
            mode,
            semantic,
            tags,
            priority,
            assignee,
        } => cmd_open(query, mode, semantic, tags, priority, assignee).await,

        Commands::Watch => cmd_watch().await,

        Commands::Doctor { repair, json, normalize_tags } => cmd_doctor(repair, json, normalize_tags).await,
        Commands::Card { action } => cmd_card(action).await,
        Commands::List { status, tags, priority, assignee, created, updated, json } => 
            cmd_list(status, tags, priority, assignee, created, updated, json).await,
        Commands::Clean { logs } => cmd_clean(logs).await,
        Commands::Logs { action } => cmd_logs(action).await,
        Commands::Upgrade => cmd_upgrade().await,
        Commands::Mcp => cmd_mcp().await,
        Commands::Search {
            query,
            mode,
            tags,
            priority,
            assignee,
            json,
            limit,
        } => cmd_search(query, mode, tags, priority, assignee, json, limit).await,
        Commands::Graph {
            format,
            output,
            stats,
            json,
        } => cmd_graph(format, output, stats, json).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(EXIT_ERROR);
    }
}

//
// Helper functions
//

/// Print error as JSON for tool integration
fn print_json_error(error: &anyhow::Error, code: &str) {
    use serde_json::json;
    
    let json_error = json!({
        "success": false,
        "error": {
            "code": code,
            "message": error.to_string(),
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&json_error).unwrap());
}

//
// Command implementations
//

async fn cmd_init() -> anyhow::Result<()> {
    use std::fs;

    let cuedeck_dir = Path::new(".cuedeck");

    // Create directory structure
    if !cuedeck_dir.exists() {
        fs::create_dir(cuedeck_dir)?;
        eprintln!("✓ Created .cuedeck/");
    } else {
        eprintln!("✓ .cuedeck/ already exists");
    }

    // Create cards/ subdirectory
    let cards_dir = cuedeck_dir.join("cards");
    if !cards_dir.exists() {
        fs::create_dir(&cards_dir)?;
        eprintln!("✓ Created .cuedeck/cards/");
    }

    // Create docs/ subdirectory
    let docs_dir = cuedeck_dir.join("docs");
    if !docs_dir.exists() {
        fs::create_dir(&docs_dir)?;
        eprintln!("✓ Created .cuedeck/docs/");
    }

    // Create default config if it doesn't exist
    let config_path = cuedeck_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = r#"# CueDeck Configuration
# See: https://docs.cuedeck.dev/config

[core]
token_limit = 32000
hash_algo = "sha256"

[parser]
ignore_patterns = ["target/", "node_modules/", ".git/"]
anchor_levels = [1, 2, 3]

[security]
secret_patterns = ["sk-.*", "ghp_.*"]

[mcp]
search_limit = 10

[author]
name = ""
email = ""

[watcher]
enabled = true
debounce_ms = 500

[cache]
cache_mode = "lazy"
memory_limit_mb = 512
"#;
        fs::write(&config_path, default_config)?;
        eprintln!("✓ Created .cuedeck/config.toml");
    } else {
        eprintln!("✓ .cuedeck/config.toml already exists");
    }

    // Append to .gitignore if it exists, create if not
    let gitignore_path = Path::new(".gitignore");
    let gitignore_entries = "\n# CueDeck\n.cuedeck/.cache/\n.cuedeck/SCENE.md\n";

    if gitignore_path.exists() {
        let content = fs::read_to_string(gitignore_path)?;
        if !content.contains(".cuedeck/.cache") {
            fs::write(gitignore_path, format!("{}{}", content, gitignore_entries))?;
            eprintln!("✓ Updated .gitignore");
        }
    } else {
        fs::write(gitignore_path, gitignore_entries)?;
        eprintln!("✓ Created .gitignore");
    }

    eprintln!("\n✅ Workspace initialized successfully!");
    Ok(())
}

async fn cmd_scene(dry_run: bool, _token_limit: Option<usize>) -> anyhow::Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Instant;

    let start = Instant::now();
    let workspace_root = Path::new(".");

    // Create spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Indexing and generating scene...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Generate scene
    let scene = cue_core::generate_scene(workspace_root)?;

    pb.finish_and_clear();

    // Count tokens (rough estimate)
    let tokens = scene.len() / 4;

    if dry_run {
        // Output to stdout
        println!("{}", scene);
    } else {
        // Copy to clipboard
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&scene)?;
    }

    // Print stats to stderr
    let elapsed = start.elapsed();
    eprintln!(
        "✓ Scene built in {}ms. {} tokens.",
        elapsed.as_millis(),
        tokens
    );

    if !dry_run {
        eprintln!("✓ Copied to clipboard");
    }

    Ok(())
}

async fn cmd_open(
    query: Option<String>,
    mode: String,
    semantic: bool,
    tags: Option<Vec<String>>,
    priority: Option<String>,
    assignee: Option<String>,
) -> anyhow::Result<()> {
    use cue_core::context::{search_workspace_with_mode, SearchFilters, SearchMode};
    use std::io::{self, Write};

    let cwd = std::env::current_dir()?;
    let query_str = query.unwrap_or_default();

    // Determine search mode: --semantic flag overrides --mode for backward compat
    let search_mode = if semantic {
        SearchMode::Semantic
    } else {
        SearchMode::parse(&mode)
    };

    // Construct filters if any provided
    let filters = if tags.is_some() || priority.is_some() || assignee.is_some() {
        Some(SearchFilters {
            tags,
            priority,
            assignee,
        })
    } else {
        None
    };

    // Log active filters to stderr
    if let Some(ref f) = filters {
        if let Some(ref t) = f.tags {
            eprintln!("🏷️  Filtering by tags: {}", t.join(", "));
        }
        if let Some(ref p) = f.priority {
            eprintln!("⚡ Filtering by priority: {}", p);
        }
        if let Some(ref a) = f.assignee {
            eprintln!("👤 Filtering by assignee: {}", a);
        }
    }

    match search_mode {
        SearchMode::Hybrid => eprintln!("🔍 Using hybrid search (semantic + keyword)..."),
        SearchMode::Semantic => eprintln!("🔍 Using semantic search..."),
        SearchMode::Keyword => eprintln!("🔍 Using keyword search..."),
    }

    let scored_results = search_workspace_with_mode(&cwd, &query_str, search_mode, filters)?;

    if scored_results.is_empty() {
        eprintln!("No results found for query: '{}'", query_str);
        return Ok(());
    }

    // Display numbered list with scores
    eprintln!("Select a file:");
    for (i, result) in scored_results.iter().enumerate() {
        eprintln!("  {}. [{:.2}] {}", i + 1, result.score, result.document.path.display());
    }

    // Get user input
    eprint!("\nEnter number (1-{}): ", scored_results.len());
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= scored_results.len() {
            let path = &scored_results[choice - 1].document.path;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());

            std::process::Command::new(&editor).arg(path).status()?;
        } else {
            eprintln!("Invalid choice");
        }
    } else {
        eprintln!("Invalid input");
    }

    Ok(())
}

async fn cmd_search(
    query: String,
    mode: String,
    tags: Option<Vec<String>>,
    priority: Option<String>,
    assignee: Option<String>,
    json: bool,
    limit: usize,
) -> anyhow::Result<()> {
    use cue_core::context::{search_workspace_with_mode, SearchFilters, SearchMode};
    use serde_json::json;

    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) if json => {
            print_json_error(&e.into(), "CWD_ERROR");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let search_mode = SearchMode::parse(&mode);

    let filters = if tags.is_some() || priority.is_some() || assignee.is_some() {
        Some(SearchFilters {
            tags,
            priority,
            assignee,
        })
    } else {
        None
    };

    let scored_results = match search_workspace_with_mode(&cwd, &query, search_mode, filters) {
        Ok(results) => results,
        Err(e) if json => {
            print_json_error(&e.into(), "SEARCH_FAILED");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let results: Vec<_> = scored_results.into_iter().take(limit).collect();

    if json {
        // JSON output for VSCode extension with real scores
        let json_results: Vec<_> = results
            .iter()
            .map(|result| {
                let preview = if let Some(ref fm) = result.document.frontmatter {
                    fm.title.clone()
                } else {
                    result.document.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string()
                };
                json!({
                    "path": result.document.path.display().to_string(),
                    "score": format!("{:.2}", result.score), // ✅ Real calculated score
                    "preview": preview,
                })
            })
            .collect();
        
        println!("{}", serde_json::to_string_pretty(&json_results)?);
    } else {
        // Human-readable output with scores
        eprintln!("Found {} results:", results.len());
        for (i, result) in results.iter().enumerate() {
            eprintln!("{}. [{:.2}] {}", 
                i + 1, 
                result.score, 
                result.document.path.display()
            );
        }
    }

    Ok(())
}

async fn cmd_doctor(repair: bool, json: bool, normalize_tags: bool) -> anyhow::Result<()> {
    use cue_core::doctor::{run_diagnostics, run_repairs, CheckStatus};

    let cwd = std::env::current_dir()?;

    if !json {
        eprintln!("🔍 Running workspace health checks...\n");
    }

    let report = run_diagnostics(&cwd)?;

    if json && !repair {
        // JSON output without repair
        let json_str = serde_json::to_string_pretty(&report)?;
        println!("{}", json_str);
        return Ok(());
    }

    // Human-readable output
    if !json {
        for check in &report.checks {
            let icon = match check.status {
                CheckStatus::Pass => "✓",
                CheckStatus::Warn => "⚠",
                CheckStatus::Fail => "✗",
            };

            let fixable_hint = if check.fixable && check.status != CheckStatus::Pass {
                " [fixable]"
            } else {
                ""
            };

            eprintln!("  {} {}: {}{}", icon, check.name, check.message, fixable_hint);

            if let Some(details) = &check.details {
                for detail in details.iter().take(5) {
                    eprintln!("      {}", detail);
                }
                if details.len() > 5 {
                    eprintln!("      ... and {} more", details.len() - 5);
                }
            }
        }

        // Print stats
        if let Some(stats) = &report.stats {
            eprintln!("\n📊 Workspace Statistics:");
            eprintln!("  Total tasks: {}", stats.total_tasks);
            eprintln!("  Total dependencies: {}", stats.total_deps);
            eprintln!("  Orphaned tasks: {}", stats.orphaned_tasks);
            eprintln!("  Max dependency depth: {}", stats.max_depth);
        }
    }

    // Attempt repairs if requested
    if repair && !report.healthy {
        if !json {
            eprintln!("\n🔧 Attempting automatic repairs...\n");
        }

        let repair_report = run_repairs(&cwd, &report, normalize_tags)?;

        // Display repair results
        if !json {
            for result in &repair_report.details {
                let icon = if result.success { "✓" } else { "✗" };
                eprintln!("  {} {}: {}", icon, result.check_name, result.message);
            }

            eprintln!("\n📋 Repair Summary:");
            eprintln!("  Attempted: {}", repair_report.total_attempted);
            eprintln!("  Successful: {}", repair_report.successful);
            eprintln!("  Failed: {}", repair_report.failed);
        }

        // Re-run diagnostics to verify repairs
        if !json {
            eprintln!("\n🔍 Re-running diagnostics to verify repairs...\n");
        }

        let final_report = run_diagnostics(&cwd)?;

        if json {
            // JSON output with repair results
            use serde_json::json;
            let output = json!({
                "initial_diagnostics": report,
                "repairs": repair_report,
                "final_diagnostics": final_report,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            // Show final status
            for check in &final_report.checks {
                let icon = match check.status {
                    CheckStatus::Pass => "✓",
                    CheckStatus::Warn => "⚠",
                    CheckStatus::Fail => "✗",
                };
                eprintln!("  {} {}: {}", icon, check.name, check.message);
            }

            eprintln!();
            if final_report.healthy {
                eprintln!("✅ All issues resolved!");
            } else {
                let failed = final_report
                    .checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Fail)
                    .count();
                let warned = final_report
                    .checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Warn)
                    .count();

                eprintln!("⚠️  Some issues remain:");
                if failed > 0 {
                    eprintln!("  {} issue(s) need manual attention", failed);
                }
                if warned > 0 {
                    eprintln!("  {} warning(s)", warned);
                }
            }
        }
    } else if !json {
        // No repair, just show summary
        eprintln!();
        if report.healthy {
            eprintln!("✅ All checks passed!");
        } else {
            let failed = report
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Fail)
                .count();
            let warned = report
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Warn)
                .count();
            let fixable = report
                .checks
                .iter()
                .filter(|c| c.fixable && c.status != CheckStatus::Pass)
                .count();

            eprintln!("❌ Health check failed:");
            if failed > 0 {
                eprintln!("  {} issue(s) need attention", failed);
            }
            if warned > 0 {
                eprintln!("  {} warning(s)", warned);
            }
            if fixable > 0 {
                eprintln!("\n💡 Tip: Run with --repair to automatically fix {} issue(s)", fixable);
            }

            std::process::exit(EXIT_ERROR);
        }
    }

    Ok(())
}

async fn cmd_card(action: CardAction) -> anyhow::Result<()> {
    match action {
        CardAction::New { title } => {
            let cwd = std::env::current_dir()?;
            let path = cue_core::tasks::create_task(&cwd, &title)?;
            eprintln!("✓ Created {}", path.display());
        }

        CardAction::Create {
            title,
            tags,
            priority,
            assignee,
            depends_on,
        } => {
            let cwd = std::env::current_dir()?;

            // Validate priority
            if !["low", "medium", "high", "critical"].contains(&priority.as_str()) {
                anyhow::bail!(
                    "Invalid priority '{}'. Must be: low, medium, high, or critical",
                    priority
                );
            }

            let path = cue_core::tasks::create_task_with_metadata(
                &cwd,
                &title,
                tags.clone(),
                Some(&priority),
                assignee.as_deref(),
                depends_on.clone(),
            )?;

            let task_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            eprintln!("✓ Created task: {} at {}", task_id, path.display());

            // Show metadata summary
            if let Some(t) = tags {
                eprintln!("  Tags: {}", t.join(", "));
            }
            eprintln!("  Priority: {}", priority);
            if let Some(a) = assignee {
                eprintln!("  Assignee: {}", a);
            }
            if let Some(deps) = depends_on {
                if !deps.is_empty() {
                    eprintln!("  Depends on: {}", deps.join(", "));
                }
            }
        }

        CardAction::Deps { id, reverse } => {
            let cwd = std::env::current_dir()?;

            if reverse {
                // Show dependents (tasks that depend on this task)
                let dependents = cue_core::tasks::get_task_dependents(&cwd, &id)?;

                if dependents.is_empty() {
                    eprintln!("No tasks depend on '{}'", id);
                } else {
                    eprintln!("Tasks depending on '{}' ({} total):", id, dependents.len());
                    for dep in dependents {
                        // Load task details
                        let task_path = cwd.join(format!(".cuedeck/cards/{}.md", dep.from_id));
                        if let Ok(doc) = cue_core::parse_file(&task_path) {
                            let title = doc
                                .frontmatter
                                .as_ref()
                                .map(|m| m.title.as_str())
                                .unwrap_or("Untitled");
                            eprintln!("  ← {}: {}", dep.from_id, title);
                        } else {
                            eprintln!("  ← {}", dep.from_id);
                        }
                    }
                }
            } else {
                // Show dependencies (tasks this task depends on)
                let dependencies = cue_core::tasks::get_task_dependencies(&cwd, &id)?;

                if dependencies.is_empty() {
                    eprintln!("Task '{}' has no dependencies", id);
                } else {
                    eprintln!(
                        "Dependencies for '{}' ({} total):",
                        id,
                        dependencies.len()
                    );
                    for dep in dependencies {
                        // Load task details
                        let task_path = cwd.join(format!(".cuedeck/cards/{}.md", dep.to_id));
                        if let Ok(doc) = cue_core::parse_file(&task_path) {
                            let title = doc
                                .frontmatter
                                .as_ref()
                                .map(|m| m.title.as_str())
                                .unwrap_or("Untitled");
                            eprintln!("  → {}: {}", dep.to_id, title);
                        } else {
                            eprintln!("  → {}", dep.to_id);
                        }
                    }
                }
            }
        }

        CardAction::Validate { id } => {
            use cue_core::task_graph::TaskGraph;
            let cwd = std::env::current_dir()?;

            if let Some(task_id) = id {
                // Validate specific task
                eprintln!("Validating task '{}'...", task_id);

                // Get dependencies for this task
                let deps = cue_core::tasks::get_task_dependencies(&cwd, &task_id)?;
                let dep_ids: Vec<String> = deps.into_iter().map(|d| d.to_id).collect();

                match cue_core::tasks::validate_task_dependencies(&cwd, &task_id, &dep_ids) {
                    Ok(_) => eprintln!("✓ Task '{}' dependencies are valid", task_id),
                    Err(e) => {
                        eprintln!("❌ Validation failed: {}", e);
                        std::process::exit(EXIT_ERROR);
                    }
                }
            } else {
                // Validate entire task graph
                eprintln!("Validating entire task dependency graph...");

                let graph = TaskGraph::from_workspace(&cwd)?;
                match graph.validate_dependencies() {
                    Ok(_) => eprintln!("✓ All task dependencies are valid (no circular dependencies)"),
                    Err(e) => {
                        eprintln!("❌ Validation failed: {}", e);
                        std::process::exit(EXIT_ERROR);
                    }
                }
            }
        }

        CardAction::Graph { format, output } => {
            use cue_core::task_graph::TaskGraph;
            use std::fs;
            let cwd = std::env::current_dir()?;

            eprintln!("Building task dependency graph...");
            let graph = TaskGraph::from_workspace(&cwd)?;

            let rendered = match format.to_lowercase().as_str() {
                "dot" => graph.to_dot(),
                "mermaid" => graph.to_mermaid(),
                "json" => graph.to_json()?,
                _ => anyhow::bail!(
                    "Invalid format '{}'. Must be: dot, mermaid, or json",
                    format
                ),
            };

            if let Some(output_path) = output {
                fs::write(&output_path, &rendered)?;
                eprintln!("✓ Task graph written to {}", output_path);
            } else {
                println!("{}", rendered);
            }
        }

        CardAction::List { status } => {
            cmd_list(status, None, None, None, None, None, false).await?;
        }

        CardAction::Edit { id } => {
            let path = format!(".cuedeck/cards/{}.md", id);
            if !Path::new(&path).exists() {
                anyhow::bail!("Card not found: {}", id);
            }

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            std::process::Command::new(&editor).arg(&path).status()?;
        }

        CardAction::Archive { id } => {
            let cwd = std::env::current_dir()?;
            let mut updates = serde_json::Map::new();
            updates.insert(
                "status".to_string(),
                serde_json::Value::String("archived".to_string()),
            );

            match cue_core::tasks::update_task(&cwd, &id, updates) {
                Ok(_) => eprintln!("✓ Archived card: {}", id),
                Err(e) => anyhow::bail!("Failed to archive card {}: {}", id, e),
            }
        }
    }

    Ok(())
}

async fn cmd_list(
    status: String,
    tags: Option<Vec<String>>,
    priority: Option<String>,
    assignee: Option<String>,
    created: Option<String>,
    updated: Option<String>,
    json: bool, // ✅ Remove underscore - no longer a stub
) -> anyhow::Result<()> {
    use cue_core::task_filters::{parse_date_filter, TaskFilters};

    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) if json => {
            print_json_error(&e.into(), "CWD_ERROR");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // Build filters
    let mut filters = TaskFilters::default();

    // Status filter
    if status != "all" {
        filters.status = Some(status.clone());
    }

    // Tags filter
    filters.tags = tags;

    // Priority filter
    filters.priority = priority.clone();

    // Assignee filter
    filters.assignee = assignee.clone();

    // Created date filter
    if let Some(created_str) = created {
        filters.created = match parse_date_filter(&created_str) {
            Ok(f) => Some(f),
            Err(e) if json => {
                print_json_error(&e.into(), "INVALID_DATE_FILTER");
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
    }

    // Updated date filter
    if let Some(updated_str) = updated {
        filters.updated = match parse_date_filter(&updated_str) {
            Ok(f) => Some(f),
            Err(e) if json => {
                print_json_error(&e.into(), "INVALID_DATE_FILTER");
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
    }

    // Use new filtered function
    let tasks = match cue_core::tasks::list_tasks_filtered(&cwd, &filters) {
        Ok(t) => t,
        Err(e) if json => {
            print_json_error(&e.into(), "LIST_FAILED");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if json {
        // ✅ JSON output for VSCode extension
        use serde_json::json;
        
        let json_tasks: Vec<_> = tasks
            .iter()
            .map(|doc| {
                let id = doc.path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let meta = doc.frontmatter.as_ref();
                
                json!({
                    "id": id,
                    "title": meta.map(|m| m.title.as_str()).unwrap_or("Untitled"),
                    "status": meta.map(|m| m.status.as_str()).unwrap_or("unknown"),
                    "priority": meta.map(|m| m.priority.as_str()).unwrap_or("medium"),
                    "assignee": meta.and_then(|m| m.assignee.as_deref()),
                    "tags": meta.and_then(|m| m.tags.as_ref()),
                    "file": doc.path.display().to_string(),
                    "line": 1, // Frontmatter is always at line 1
                    "created": meta.and_then(|m| m.created.as_ref()),
                    "updated": meta.and_then(|m| m.updated.as_ref()),
                    "dependsOn": meta.and_then(|m| m.depends_on.as_ref()),
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&json_tasks)?);
    } else {
        // Human-readable output
        // Display filter info
        let mut filter_parts = vec![format!("status={}", status)];
        if let Some(ref t) = filters.tags {
            filter_parts.push(format!("tags={}", t.join(",")));
        }
        if let Some(ref p) = priority {
            filter_parts.push(format!("priority={}", p));
        }
        if let Some(ref a) = assignee {
            filter_parts.push(format!("assignee={}", a));
        }

        eprintln!("Cards ({}):", filter_parts.join(", "));
        eprintln!(
            "{:<10} {:<30} {:<15} {:<10}",
            "ID", "Title", "Status", "Priority"
        );
        eprintln!("{}", "-".repeat(70));

        for doc in tasks {
            // ID from filename
            let id = doc
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let meta = doc.frontmatter.unwrap_or(cue_common::CardMetadata {
                title: "Untitled".to_string(),
                status: "unknown".to_string(),
                assignee: None,
                priority: "medium".to_string(),
                tags: None,
                created: None,
                updated: None,
                depends_on: None,
            });

            eprintln!(
                "{:<10} {:<30} {:<15} {:<10}",
                id,
                truncate(&meta.title, 28),
                meta.status,
                meta.priority
            );
        }
    }

    Ok(())
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}..", &s[..max_width - 2])
    } else {
        s.to_string()
    }
}

async fn cmd_clean(remove_logs: bool) -> anyhow::Result<()> {
    use std::fs;

    let cache_dir = Path::new(".cuedeck/.cache");
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
        eprintln!("✓ Removed .cuedeck/.cache/");
    }

    if remove_logs {
        let logs_dir = Path::new(".cuedeck/logs");
        if logs_dir.exists() {
            fs::remove_dir_all(logs_dir)?;
            eprintln!("✓ Removed .cuedeck/logs/");
        }
    }

    eprintln!("✅ Cache cleared!");
    Ok(())
}

async fn cmd_logs(action: LogAction) -> anyhow::Result<()> {
    match action {
        LogAction::Archive => {
            use chrono::Local;
            use std::fs;

            eprintln!("✓ Archiving logs...");

            let logs_dir = Path::new(".cuedeck/logs");
            if logs_dir.exists() {
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                let archive_dir = logs_dir.join("archive").join(timestamp.to_string());

                fs::create_dir_all(&archive_dir)?;

                for entry in fs::read_dir(logs_dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
                        let file_name = path.file_name().unwrap();
                        fs::rename(&path, archive_dir.join(file_name))?;
                    }
                }

                eprintln!("✅ Logs archived to {:?}", archive_dir);
            } else {
                eprintln!("⚠ No logs directory found.");
            }
        }

        LogAction::Clear => {
            use std::fs;
            let logs_dir = Path::new(".cuedeck/logs");
            if logs_dir.exists() {
                fs::remove_dir_all(logs_dir)?;
                fs::create_dir(logs_dir)?;
                eprintln!("✅ Logs cleared");
            }
        }
    }

    Ok(())
}

async fn cmd_upgrade() -> anyhow::Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    use self_update::cargo_crate_version;
    use semver::Version;
    use std::time::Duration;

    eprintln!("✓ Checking for updates...");

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    tracing::info!("Current version: {}", current_version);

    // Create progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Fetching latest release from GitHub...");
    pb.enable_steady_tick(Duration::from_millis(80));

    // Use self_update to check and perform update
    let update_result = self_update::backends::github::Update::configure()
        .repo_owner("TCTri205")
        .repo_name("CueDeck")
        .bin_name("cue")
        .current_version(cargo_crate_version!())
        .no_confirm(true)
        .build()?
        .update();

    pb.finish_and_clear();

    match update_result {
        Ok(status) => match status {
            self_update::Status::UpToDate(v) => {
                eprintln!("✓ You are using the latest version ({})", v);
            }
            self_update::Status::Updated(v) => {
                eprintln!("✅ Successfully updated to version {}", v);
                eprintln!("   Please restart the application for changes to take effect.");

                #[cfg(target_os = "windows")]
                eprintln!(
                    "   Note: On Windows, the update will complete on next application start."
                );

                tracing::info!("Successfully updated to version {}", v);
            }
        },
        Err(e) => {
            let err_msg = e.to_string();

            if err_msg.contains("rate limit") {
                eprintln!("⚠ GitHub API rate limit reached. Please try again later.");
                tracing::warn!("GitHub rate limit: {}", e);
            } else if err_msg.contains("404") || err_msg.contains("not found") {
                eprintln!("⚠ No release found at https://github.com/TCTri205/CueDeck/releases");
                eprintln!("  (This is expected if the repository has no releases yet)");
                tracing::warn!("No GitHub release found: {}", e);
            } else if err_msg.to_lowercase().contains("network") || err_msg.contains("connect") {
                eprintln!("⚠ Network error. Check your internet connection.");
                tracing::error!("Network error during update: {}", e);
            } else {
                eprintln!("⚠ Update failed: {}", e);
                tracing::error!("Update error: {}", e);
            }

            eprintln!("  Fallback: Download manually from https://github.com/TCTri205/CueDeck/releases/latest");
        }
    }

    Ok(())
}

async fn cmd_watch() -> anyhow::Result<()> {
    use arboard::Clipboard;
    use cue_core::engine::CueEngine;
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};

    eprintln!("✓ Starting CueDeck Watcher...");
    eprintln!("  Watching .cuedeck/ and src/ for changes...");

    let root = std::env::current_dir()?;

    // Initialize Engine
    use indicatif::{ProgressBar, ProgressStyle};
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Initializing engine...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let mut engine = CueEngine::new(&root).map_err(|e| anyhow::anyhow!(e))?;
    pb.finish_and_clear();

    // Initial build
    eprintln!("  Initial build...");
    match engine.render() {
        Ok(scene) => {
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Err(e) = clipboard.set_text(&scene) {
                    eprintln!("⚠ Failed to update clipboard: {}", e);
                } else {
                    eprintln!("✓ Clipboard updated ({} tokens)", scene.len() / 4);
                }
            } else {
                eprintln!("⚠ Clipboard unavailable");
            }
        }
        Err(e) => eprintln!("⚠ Initial build failed: {}", e),
    }

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&root, RecursiveMode::Recursive)?;

    let mut last_update = Instant::now();
    let debounce_duration = Duration::from_millis(500);

    // Event loop
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Filter relevant events
                let relevant_paths: Vec<_> = event
                    .paths
                    .into_iter()
                    .filter(|p| {
                        let s = p.to_string_lossy();
                        !s.contains("target")
                            && !s.contains(".git")
                            && !s.contains(".cuedeck\\cache")
                            && !s.contains(".cuedeck/cache")
                            && !s.ends_with("SCENE.md")
                            && (s.ends_with(".md") || s.ends_with("cue.toml"))
                    })
                    .collect();

                if !relevant_paths.is_empty() {
                    // Debounce
                    if last_update.elapsed() > debounce_duration {
                        eprintln!("⟳ Change detected, updating...");

                        // Update engine state
                        for path in relevant_paths {
                            if path.exists() {
                                if let Err(e) = engine.update_file(&path) {
                                    tracing::warn!("Failed to update {:?}: {}", path, e);
                                }
                            } else {
                                engine.remove_file(&path);
                            }
                        }

                        // Re-render
                        match engine.render() {
                            Ok(scene) => match Clipboard::new() {
                                Ok(mut clipboard) => {
                                    if let Err(e) = clipboard.set_text(&scene) {
                                        tracing::error!("Clipboard error: {}", e);
                                    } else {
                                        eprintln!(
                                            "✓ Clipboard updated ({} tokens)",
                                            scene.len() / 4
                                        );
                                    }
                                }
                                Err(e) => eprintln!("⚠ Clipboard unavailable: {}", e),
                            },
                            Err(e) => eprintln!("⚠ Build failed: {}", e),
                        }

                        last_update = Instant::now();
                    }
                }
            }
            Ok(Err(e)) => eprintln!("⚠ Watcher error: {}", e),
            Err(_) => break, // Channel closed
        }
    }

    Ok(())
}

async fn cmd_graph(format: String, output: Option<String>, stats: bool, json: bool) -> anyhow::Result<()> {
    use cue_core::graph::DependencyGraph;
    use std::fs;

    let cwd = std::env::current_dir()?;

    // Collect all markdown documents
    let mut all_docs = Vec::new();

    for entry in walkdir::WalkDir::new(&cwd)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !(name == "node_modules" || name == ".git" || name == "target" || name == "dist")
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    match cue_core::parse_file(entry.path()) {
                        Ok(doc) => all_docs.push(doc),
                        Err(e) => tracing::warn!("Failed to parse {:?}: {}", entry.path(), e),
                    }
                }
            }
        }
    }

    if all_docs.is_empty() {
        eprintln!("⚠ No markdown files found in workspace");
        return Ok(());
    }

    // Build dependency graph
    let graph = match DependencyGraph::build(&all_docs) {
        Ok(g) => g,
        Err(e) if json => {
            print_json_error(&e.into(), "GRAPH_BUILD_FAILED");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if json {
        // ✅ JSON export for VSCode extension
        let export = graph.to_json_export();
        let json_str = serde_json::to_string_pretty(&export)?;
        
        if let Some(output_path) = output {
            fs::write(&output_path, &json_str)?;
            eprintln!("✓ Graph JSON written to {}", output_path);
        } else {
            println!("{}", json_str);
        }
    } else {
        // Existing visualization rendering
        // Show statistics if requested
        if stats {
            let graph_stats = graph.stats();
            eprintln!("Graph Statistics:");
            eprintln!("  Nodes: {}", graph_stats.node_count);
            eprintln!("  Edges: {}", graph_stats.edge_count);
            eprintln!(
                "  Cycles: {}",
                if graph_stats.has_cycles { "Yes" } else { "No" }
            );

            if graph_stats.has_cycles {
                if let Some(cycle) = graph.detect_cycle() {
                    let cycle_str: Vec<String> = cycle
                        .iter()
                        .map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect();
                    eprintln!("  Cycle: {}", cycle_str.join(" → "));
                }
            }

            let orphans = graph.orphans();
            eprintln!("  Orphans: {} documents", orphans.len());
            eprintln!();
        }

        use cue_core::graph_viz::{render, GraphFormat};

        // Parse format
        let graph_format: GraphFormat = format.parse().map_err(|e: String| {
            anyhow::anyhow!("Invalid format: {}. Use: mermaid, dot, ascii, json", e)
        })?;

        // Render graph
        let rendered = render(&graph, graph_format);

        // Output to file or stdout
        if let Some(output_path) = output {
            fs::write(&output_path, &rendered)?;
            eprintln!("✓ Graph written to {}", output_path);
        } else {
            println!("{}", rendered);
        }
    }

    Ok(())
}

async fn cmd_mcp() -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader};

    // CRITICAL: Log to stderr, NOT stdout
    // stdout is reserved EXCLUSIVELY for JSON-RPC responses
    eprintln!("✓ MCP server started (reading from stdin)");

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<cue_mcp::JsonRpcRequest>(&line) {
            Ok(request) => {
                if let Some(response) = cue_mcp::handle_request(request).await {
                    // Write JSON-RPC response to stdout ONLY
                    println!("{}", serde_json::to_string(&response)?);
                }
            }
            Err(e) => {
                // Parse errors must return JSON-RPC error on stdout, not stderr
                let error_response = cue_mcp::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(cue_mcp::JsonRpcError {
                        code: -32700, // Parse error
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                println!("{}", serde_json::to_string(&error_response)?);
                // Also log to stderr for debugging
                eprintln!("Parse error: {}", e);
            }
        }
    }

    Ok(())
}
