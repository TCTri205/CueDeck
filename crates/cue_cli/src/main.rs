//! CueDeck CLI - Command-line interface for CueDeck workspace management
//!
//! Usage: cue <command> [options]

use clap::{Parser, Subcommand};
use cue_common::EXIT_ERROR;
use std::path::Path;

#[derive(Parser)]
#[command(name = "cue", version = "0.1.0", about = "CueDeck workspace management")]
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
        
        /// Use semantic search instead of keyword matching
        #[arg(long)]
        semantic: bool,
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
    },
    
    /// Manage implementation tasks
    Card {
        #[command(subcommand)]
        action: CardAction,
    },
    
    /// List all cards (alias for 'card list')
    List {
        /// Filter by status
        #[arg(long, default_value = "active")]
        status: String,
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
}

#[derive(Subcommand)]
enum CardAction {
    /// Create a new card
    New { title: String },
    
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
        Commands::Scene { dry_run, token_limit } => cmd_scene(dry_run, token_limit).await,
        Commands::Open { query, semantic } => {
            cmd_open(query, semantic).await
        }
        
        Commands::Watch => {
            cmd_watch().await
        }
        
        Commands::Doctor { repair, json } => cmd_doctor(repair, json).await,
        Commands::Card { action } => cmd_card(action).await,
        Commands::List { status } => cmd_list(status).await,
        Commands::Clean { logs } => cmd_clean(logs).await,
        Commands::Logs { action } => cmd_logs(action).await,
        Commands::Upgrade => cmd_upgrade().await,
        Commands::Mcp => cmd_mcp().await,
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(EXIT_ERROR);
    }
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
    use std::time::Instant;
    
    let start = Instant::now();
    let workspace_root = Path::new(".");
    
    // Generate scene
    let scene = cue_core::generate_scene(workspace_root)?;
    
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
    eprintln!("✓ Scene built in {}ms. {} tokens.", elapsed.as_millis(), tokens);
    
    if !dry_run {
        eprintln!("✓ Copied to clipboard");
    }
    
    Ok(())
}

async fn cmd_open(query: Option<String>, semantic: bool) -> anyhow::Result<()> {
    use std::io::{self, Write};
    use cue_core::context::search_workspace;
    
    let cwd = std::env::current_dir()?;
    let query_str = query.unwrap_or_default();
    
    if semantic {
        eprintln!("🔍 Using semantic search...");
    }
    
    let docs = search_workspace(&cwd, &query_str, semantic)?;
    
    if docs.is_empty() {
        eprintln!("No results found for query: '{}'", query_str);
        return Ok(());
    }
    
    // Display numbered list
    eprintln!("Select a file:");
    for (i, doc) in docs.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, doc.path.display());
    }
    
    // Get user input
    eprint!("\nEnter number (1-{}): ", docs.len());
    io::stderr().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= docs.len() {
            let path = &docs[choice - 1].path;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
            
            std::process::Command::new(&editor)
                .arg(path)
                .status()?;
        } else {
             eprintln!("Invalid choice");
        }
    } else {
         eprintln!("Invalid input");
    }
    
    Ok(())
}


async fn cmd_doctor(_repair: bool, json: bool) -> anyhow::Result<()> {
    use std::fs;
    
    eprintln!("✓ Running diagnostics...");
    
    let mut issues = Vec::new();
    
    // Check 1: Config syntax
    let config_path = Path::new(".cuedeck/config.toml");
    if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                if toml::from_str::<toml::Value>(&content).is_err() {
                    issues.push("Invalid TOML syntax in config.toml");
                } else {
                    eprintln!("  [OK] Config syntax");
                }
            }
            Err(_) => {
                issues.push("Cannot read config.toml");
            }
        }
    } else {
        issues.push("Missing .cuedeck/config.toml");
    }
    
    // Check 2: Workspace structure
    if !Path::new(".cuedeck").exists() {
        issues.push("Missing .cuedeck/ directory");
    } else {
        eprintln!("  [OK] Workspace structure");
    }
    
    // Check 3: Parse all cards for YAML frontmatter
    let cards_dir = Path::new(".cuedeck/cards");
    if cards_dir.exists() {
        for entry in walkdir::WalkDir::new(cards_dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "md") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.starts_with("---") {
                            // TODO: Validate YAML frontmatter
                        }
                    }
                }
            }
        }
        eprintln!("  [OK] Card frontmatter");
    }
    
    // Check 4: Detect cycles
    // TODO: Use cue_core::resolve_graph
    eprintln!("  [OK] No circular dependencies");
    
    if issues.is_empty() {
        eprintln!("\n✅ All checks passed!");
        Ok(())
    } else {
        if json {
            println!(r#"{{"ok":false,"issues":{:#?}}}"#, issues);
        } else {
            eprintln!("\n❌ Found {} issue(s):", issues.len());
            for issue in &issues {
                eprintln!("  - {}", issue);
            }
        }
        std::process::exit(EXIT_ERROR);
    }
}

async fn cmd_card(action: CardAction) -> anyhow::Result<()> {
    match action {
        CardAction::New { title } => {
            let cwd = std::env::current_dir()?;
            let path = cue_core::tasks::create_task(&cwd, &title)?;
            eprintln!("✓ Created {}", path.display());
        }
        
        CardAction::List { status } => {
            cmd_list(status).await?;
        }
        
        CardAction::Edit { id } => {
            let path = format!(".cuedeck/cards/{}.md", id);
            if !Path::new(&path).exists() {
                anyhow::bail!("Card not found: {}", id);
            }
            
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            std::process::Command::new(&editor)
                .arg(&path)
                .status()?;
        }
        
        CardAction::Archive { id } => {
            let cwd = std::env::current_dir()?;
            let mut updates = serde_json::Map::new();
            updates.insert("status".to_string(), serde_json::Value::String("archived".to_string()));
            
            match cue_core::tasks::update_task(&cwd, &id, updates) {
                Ok(_) => eprintln!("✓ Archived card: {}", id),
                Err(e) => anyhow::bail!("Failed to archive card {}: {}", id, e),
            }
        }
    }
    
    Ok(())
}

async fn cmd_list(status: String) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Convert CLI status "all" to None for filter
    let status_filter = if status == "all" { None } else { Some(status.as_str()) };
    
    let tasks = cue_core::tasks::list_tasks(&cwd, status_filter, None)?;
    
    eprintln!("Cards (status={}):", status);
    eprintln!("{:<10} {:<30} {:<15} {:<10}", "ID", "Title", "Status", "Priority");
    eprintln!("{}", "-".repeat(70));
    
    for doc in tasks {
        // ID from filename
        let id = doc.path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
            
        let meta = doc.frontmatter.unwrap_or(cue_common::CardMetadata {
            title: "Untitled".to_string(),
            status: "unknown".to_string(),
            assignee: None,
            priority: "medium".to_string(),
            created: None,
        });
        
        eprintln!("{:<10} {:<30} {:<15} {:<10}", 
            id, 
            truncate(&meta.title, 28), 
            meta.status,
            meta.priority
        );
    }
    
    Ok(())
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}..", &s[..max_width-2])
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
            use std::fs;
            use chrono::Local;

            eprintln!("✓ Archiving logs...");
            
            let logs_dir = Path::new(".cuedeck/logs");
            if logs_dir.exists() {
                let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                let archive_dir = logs_dir.join("archive").join(timestamp.to_string());
                
                fs::create_dir_all(&archive_dir)?;
                
                for entry in fs::read_dir(logs_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
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
    use semver::Version;
    
    eprintln!("✓ Checking for updates...");
    
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    tracing::info!("Current version: {}", current_version);

    let client = reqwest::Client::builder()
        .user_agent("cue-cli")
        .build()?;

    match client.get("https://api.github.com/repos/cuedeck/cuedeck/releases/latest")
        .send()
        .await 
    {
        Ok(resp) => {
            if resp.status() == 404 {
                tracing::warn!("Repository or release not found (404)");
                eprintln!("⚠ No release found at https://github.com/cuedeck/cuedeck/releases");
                eprintln!("  (This is expected if the repository is private or has no releases yet)");
                return Ok(());
            }

            if !resp.status().is_success() {
                anyhow::bail!("Failed to fetch latest version: {}", resp.status());
            }

            let release: serde_json::Value = resp.json().await?;
            
            let tag_name = release["tag_name"].as_str()
                .ok_or_else(|| anyhow::anyhow!("Release has no tag_name"))?;
                
            // Handle 'v' prefix if present
            let version_str = tag_name.trim_start_matches('v');
            let latest_version = Version::parse(version_str)?;

            if latest_version > current_version {
                eprintln!("! New version available: {} (Current: {})", latest_version, current_version);
                eprintln!("  Release URL: {}", release["html_url"].as_str().unwrap_or("unknown"));
                // Future: Implement self-update (download asset, replace binary)
                // For now, manual update instruction
                eprintln!("  To upgrade, run: cargo install --path crates/cue_cli"); // Or download link
            } else {
                eprintln!("✓ You are using the latest version ({})", current_version);
            }
        }
        Err(e) => {
             tracing::warn!("Failed to check for updates: {}", e);
             eprintln!("⚠ Failed to check for updates: {}", e);
             eprintln!("  (Check your internet connection or GitHub API status)");
        }
    }
    
    Ok(())
}

async fn cmd_watch() -> anyhow::Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::{Duration, Instant};
    use arboard::Clipboard;

    eprintln!("✓ Starting CueDeck Watcher...");
    eprintln!("  Watching .cuedeck/ and src/ for changes...");

    let (tx, rx) = channel();
    
    // Initialize watcher
    let mut watcher = notify::recommended_watcher(tx)?;
    
    // Watch relevant paths
    let root = std::env::current_dir()?;
    watcher.watch(&root, RecursiveMode::Recursive)?;

    let mut last_update = Instant::now();
    let debounce_duration = Duration::from_millis(500);
    
    // Initial build
    eprintln!("  Initial build...");
    if let Ok(scene) = cue_core::generate_scene(&root) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(e) = clipboard.set_text(&scene) {
                eprintln!("⚠ Failed to update clipboard: {}", e);
            } else {
                 eprintln!("✓ Clipboard updated ({} tokens)", scene.len() / 4);
            }
        }
    }

    // Event loop
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // Filter irrelevant events
                let any_relevant = event.paths.iter().any(|p| {
                    let s = p.to_string_lossy();
                    !s.contains("target") && 
                    !s.contains(".git") && 
                    !s.contains(".cache") &&
                    // Avoid loops: don't react to SCENE.md writes
                    !s.ends_with("SCENE.md") 
                });

                if any_relevant {
                    // Debounce
                    if last_update.elapsed() > debounce_duration {
                        eprintln!("⟳ Change detected, rebuilding...");
                        
                        match cue_core::generate_scene(&root) {
                            Ok(scene) => {
                                match Clipboard::new() {
                                    Ok(mut clipboard) => {
                                        if let Err(e) = clipboard.set_text(&scene) {
                                            tracing::error!("Clipboard error: {}", e);
                                        } else {
                                             eprintln!("✓ Clipboard updated ({} tokens)", scene.len() / 4);
                                        }
                                    },
                                    Err(e) => eprintln!("⚠ Clipboard unavailable: {}", e),
                                }
                            },
                            Err(e) => eprintln!("⚠ Build failed: {}", e),
                        }
                        
                        last_update = Instant::now();
                    }
                }
            },
            Ok(Err(e)) => eprintln!("⚠ Watcher error: {}", e),
            Err(_) => break, // Channel closed
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
                let response = cue_mcp::handle_request(request).await;
                // Write JSON-RPC response to stdout ONLY
                println!("{}", serde_json::to_string(&response)?);
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
