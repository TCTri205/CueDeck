
async fn cmd_graph(format: String, output: Option<String>, stats: bool) -> anyhow::Result<()> {
    use std::fs;
    use cue_core::graph::DependencyGraph;
    use cue_core::graph_viz::{GraphFormat, render};
    
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
    {
        if let Ok(entry) = entry {
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
    }
    
    if all_docs.is_empty() {
        eprintln!("⚠ No markdown files found in workspace");
        return Ok(());
    }
    
    // Build dependency graph
    let graph = DependencyGraph::build(&all_docs)?;
    
    // Show statistics if requested
    if stats {
        let graph_stats = graph.stats();
        eprintln!("Graph Statistics:");
        eprintln!("  Nodes: {}", graph_stats.node_count);
        eprintln!("  Edges: {}", graph_stats.edge_count);
        eprintln!("  Cycles: {}", if graph_stats.has_cycles { "Yes" } else { "No" });
        
        if graph_stats.has_cycles {
            if let Some(cycle) = graph.detect_cycle() {
                let cycle_str: Vec<String> = cycle.iter()
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                    .collect();
                eprintln!("  Cycle: {}", cycle_str.join(" → "));
            }
        }
        
        let orphans = graph.orphans();
        eprintln!("  Orphans: {} documents", orphans.len());
        eprintln!();
    }
    
    // Parse format
    let graph_format = GraphFormat::from_str(&format)
        .ok_or_else(|| anyhow::anyhow!("Invalid format: '{}'. Use: mermaid, dot, ascii, json", format))?;
    
    // Render graph
    let rendered = render(&graph, graph_format);
    
    // Output to file or stdout
    if let Some(output_path) = output {
        fs::write(&output_path, &rendered)?;
        eprintln!("✓ Graph written to {}", output_path);
    } else {
        println!("{}", rendered);
    }
    
    Ok(())
}
