// Type definitions for CueDeck extension

/**
 * Result from `cue search --json` command
 */
export interface SearchResult {
    path: string;
    score: number;
    preview: string;
}

/**
 * Result from `cue task list --json` command
 */
export interface Task {
    id: string;
    title: string;
    status: 'todo' | 'in_progress' | 'done';
    priority?: 'low' | 'medium' | 'high';
    file: string;
    line: number;
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
