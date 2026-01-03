import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';
import { SearchResult, Task, CommandResult } from './types';

const execAsync = promisify(exec);

/**
 * Client for interacting with CueDeck CLI
 */
export class CuedeckClient {
    private cliPath: string;

    constructor(cliPath: string = 'cue') {
        this.cliPath = cliPath;
    }

    /**
     * Check if CLI is available
     */
    async isAvailable(): Promise<boolean> {
        try {
            await execAsync(`${this.cliPath} --version`);
            return true;
        } catch {
            return false;
        }
    }

    /**
     * Search documents
     */
    async search(query: string): Promise<CommandResult<SearchResult[]>> {
        try {
            const { stdout } = await execAsync(
                `${this.cliPath} search "${query}" --json`
            );
            const data = JSON.parse(stdout) as SearchResult[];
            return { success: true, data };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }

    /**
     * List tasks
     */
    async listTasks(): Promise<CommandResult<Task[]>> {
        try {
            const { stdout } = await execAsync(`${this.cliPath} task list --json`);
            const data = JSON.parse(stdout) as Task[];
            return { success: true, data };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }

    /**
     * Export graph data
     */
    async exportGraph(): Promise<CommandResult<any>> {
        try {
            const { stdout } = await execAsync(`${this.cliPath} graph export --json`);
            const data = JSON.parse(stdout);
            return { success: true, data };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error',
            };
        }
    }
}
