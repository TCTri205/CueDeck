// Type definitions for CueDeck extension

/**
 * Result from `cue search --json` command
 */
export interface SearchResult {
    path: string;
    score: string; // CLI returns formatted string like "0.40"
    preview: string;
}

/**
 * Result from `cue task list --json` command
 */
export interface Task {
    id: string;
    title: string;
    status: string; // 'todo' | 'active' | 'done' | 'archived'
    priority: string; // 'low' | 'medium' | 'high' | 'critical'
    assignee?: string | null;
    tags?: string[] | null;
    file: string;
    line: number;
    created?: string | null;
    updated?: string | null;
    dependsOn?: string[] | null;
}

/**
 * Configuration for CueDeck extension
 */
export interface CuedeckConfig {
    cliPath: string;
    enableSync: boolean;
}

/**
 * CLI command result
 */
export interface CommandResult<T> {
    success: boolean;
    data?: T;
    error?: string;
}

/**
 * Graph node structure
 */
export interface GraphNode {
    id: number;
    path: string;
    name: string;
    type?: string;
    metadata?: {
        status?: string;
        priority?: string;
    };
}

/**
 * Graph edge structure
 */
export interface GraphEdge {
    from: number;
    to: number;
}

/**
 * Graph statistics
 */
export interface GraphStats {
    node_count: number;
    edge_count: number;
    has_cycles: boolean;
}

/**
 * Full graph data structure
 */
export interface GraphData {
    nodes: GraphNode[];
    edges: GraphEdge[];
    stats: GraphStats;
}

/**
 * Task filter options
 */
export interface TaskFilters {
    priority: 'all' | 'low' | 'medium' | 'high' | 'critical';
    assignee: string; // 'all' or specific assignee name
    tags: string[];   // empty array = all tags
}

/**
 * Task sort options
 */
export type SortCriteria = 'priority' | 'created' | 'updated' | 'title';
export type SortDirection = 'asc' | 'desc';

export interface SortOptions {
    by: SortCriteria;
    direction: SortDirection;
}
