use cue_core::task_graph::TaskGraph;

#[test]
fn test_task_graph_basic() {
    let mut graph = TaskGraph::new();
    
    // Add tasks
    graph.add_task("task1");
    graph.add_task("task2");
    graph.add_task("task3");
    
    // Add dependencies: task1 -> task2 -> task3
    // Note: add_dependency doesn't validate automatically
    graph.add_dependency("task1", "task2").unwrap();
    graph.add_dependency("task2", "task3").unwrap();
    
    // Verify structure
    let deps1 = graph.get_dependencies("task1");
    assert_eq!(deps1.len(), 1);
    assert_eq!(deps1[0], "task2");
    
    let deps2 = graph.get_dependencies("task2");
    assert_eq!(deps2.len(), 1);
    assert_eq!(deps2[0], "task3");
}

#[test]
fn test_cycle_detection() {
    let mut graph = TaskGraph::new();
    
    graph.add_task("A");
    graph.add_task("B");
    graph.add_task("C");
    
    // A -> B -> C
    graph.add_dependency("A", "B").unwrap();
    graph.add_dependency("B", "C").unwrap();
    
    // Check if adding C -> A would create a cycle
    assert!(graph.would_create_cycle("C", "A"));
    
    // Actually add it to test find_cycle (add_dependency allows cycles)
    graph.add_dependency("C", "A").unwrap();
    
    // Now validate should fail
    assert!(graph.validate_dependencies().is_err());
}

#[test]
fn test_visualization_formats() {
    let mut graph = TaskGraph::new();
    graph.add_task("init");
    graph.add_task("build");
    graph.add_task("deploy");
    
    graph.add_dependency("build", "init").unwrap();
    graph.add_dependency("deploy", "build").unwrap();
    
    // DOT format
    let dot = graph.to_dot();
    assert!(dot.contains("digraph TaskGraph"));
    assert!(dot.contains("\"build\" -> \"init\""));
    assert!(dot.contains("\"deploy\" -> \"build\""));
    
    // Mermaid format
    let mermaid = graph.to_mermaid();
    assert!(mermaid.contains("graph LR"));
    assert!(mermaid.contains("build[\"build\"] --> init[\"init\"]"));
    
    // JSON format
    let json = graph.to_json().unwrap();
    assert!(json.contains("\"nodes\":"));
    assert!(json.contains("\"edges\":"));
    assert!(json.contains("\"from\": \"deploy\""));
}
