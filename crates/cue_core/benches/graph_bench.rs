use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cue_core::graph::DependencyGraph;
use cue_common::Document;
use std::path::PathBuf;

/// Helper to create test documents with links
fn create_test_documents(count: usize) -> Vec<Document> {
    (0..count)
        .map(|i| {
            let path = PathBuf::from(format!("doc{:04}.md", i));
            
            // Create realistic link patterns:
            // - Each doc links to 2-3 other docs
            // - Some docs have no links (10%)
            // - Some docs are heavily linked (hubs)
            let links = if i % 10 == 0 {
                // 10% orphans
                vec![]
            } else if i % 20 == 0 {
                // 5% hubs (link to many)
                (0..10)
                    .map(|j| format!("doc{:04}.md", (i + j * 7) % count))
                    .collect()
            } else {
                // Regular docs (link to 2-3)
                vec![
                    format!("doc{:04}.md", (i + 1) % count),
                    format!("doc{:04}.md", (i + 3) % count),
                    if i % 3 == 0 {
                        format!("doc{:04}.md", (i + 7) % count)
                    } else {
                        String::new()
                    },
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect()
            };

            Document {
                path,
                frontmatter: Some(cue_common::CardMetadata {
                    title: format!("Document {}", i),
                    status: "todo".to_string(),
                    assignee: None,
                    priority: if i % 3 == 0 {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    tags: Some(vec![format!("tag{}", i % 5)]),
                    created: None,
                    updated: None,
                    depends_on: None,
                }),
                hash: format!("hash{}", i),
                tokens: 100 + (i % 50),
                anchors: vec![],
                links,
            }
        })
        .collect()
}

fn bench_graph_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_build");

    // Benchmark with 100 documents
    let docs_100 = create_test_documents(100);
    group.bench_with_input(BenchmarkId::new("serial", 100), &docs_100, |b, docs| {
        b.iter(|| {
            let _ = DependencyGraph::build(black_box(docs)).unwrap();
        });
    });

    // Benchmark with 500 documents
    let docs_500 = create_test_documents(500);
    group.bench_with_input(BenchmarkId::new("serial", 500), &docs_500, |b, docs| {
        b.iter(|| {
            let _ = DependencyGraph::build(black_box(docs)).unwrap();
        });
    });

    // Benchmark with 1000 documents
    group.sample_size(10); // Fewer samples for large benchmark
    let docs_1000 = create_test_documents(1000);
    group.bench_with_input(BenchmarkId::new("serial", 1000), &docs_1000, |b, docs| {
        b.iter(|| {
            let _ = DependencyGraph::build(black_box(docs)).unwrap();
        });
    });

    group.finish();
}

fn bench_graph_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_operations");

    let docs = create_test_documents(500);
    let graph = DependencyGraph::build(&docs).unwrap();

    // Benchmark cycle detection
    group.bench_function("cycle_detection", |b| {
        b.iter(|| {
            let _ = black_box(&graph).detect_cycle();
        });
    });

    // Benchmark topological sort
    group.bench_function("topological_sort", |b| {
        b.iter(|| {
            let _ = black_box(&graph).sort_topological();
        });
    });

    // Benchmark orphan detection
    group.bench_function("find_orphans", |b| {
        b.iter(|| {
            let _ = black_box(&graph).orphans();
        });
    });

    // Benchmark stats
    group.bench_function("get_stats", |b| {
        b.iter(|| {
            let _ = black_box(&graph).stats();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_graph_build, bench_graph_operations);
criterion_main!(benches);
