import * as vscode from 'vscode';
import { CuedeckClient } from '../cuedeckClient';
import { Task, TaskFilters, SortOptions } from '../types';
import { TaskTreeItem } from './TaskTreeItem';

/**
 * Tree data provider for the CueDeck tasks sidebar
 * Displays tasks grouped by status (todo/active/done/archived)
 */
export class TaskTreeProvider implements vscode.TreeDataProvider<TaskTreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<TaskTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private tasks: Task[] = [];
    private isLoading = false;
    private searchFilter: string = '';
    private filters: TaskFilters = {
        priority: 'all',
        assignee: 'all',
        tags: []
    };
    private sortOptions: SortOptions = {
        by: 'priority',
        direction: 'desc'
    };

    constructor(private client: CuedeckClient) { }

    /**
     * Refresh the tree view
     */
    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    /**
     * Set search filter and refresh view
     */
    setSearchFilter(filter: string): void {
        this.searchFilter = filter.toLowerCase().trim();
        this.refresh();
    }

    /**
     * Set task filters and refresh view
     */
    setFilters(filters: Partial<TaskFilters>): void {
        this.filters = { ...this.filters, ...filters };
        this.refresh();
    }

    /**
     * Clear all filters
     */
    clearFilters(): void {
        this.filters = {
            priority: 'all',
            assignee: 'all',
            tags: []
        };
        this.refresh();
    }

    /**
     * Get count of active filters
     */
    getActiveFilterCount(): number {
        let count = 0;
        if (this.filters.priority !== 'all') count++;
        if (this.filters.assignee !== 'all') count++;
        if (this.filters.tags.length > 0) count++;
        return count;
    }

    /**
     * Set sort options and refresh view
     */
    setSortOptions(options: SortOptions): void {
        this.sortOptions = options;
        this.refresh();
    }

    /**
     * Get tree item for display
     */
    getTreeItem(element: TaskTreeItem): vscode.TreeItem {
        return element;
    }

    /**
     * Get children of a tree item
     * - If element is undefined, return status groups (root level)
     * - If element is a status group, return tasks in that status
     */
    async getChildren(element?: TaskTreeItem): Promise<TaskTreeItem[]> {
        // Check if workspace is available
        if (!vscode.workspace.workspaceFolders) {
            return [];
        }

        if (!element) {
            // Root level: return status groups
            await this.loadTasks();
            return this.getStatusGroups();
        } else if (element.isGroup) {
            // Status group: return tasks in that status
            const status = this.getStatusFromLabel(element.label as string);
            return this.getTasksForStatus(status);
        }

        return [];
    }

    /**
     * Load all tasks from CLI
     */
    private async loadTasks(): Promise<void> {
        if (this.isLoading) {
            return;
        }

        this.isLoading = true;
        try {
            const result = await this.client.listTasks('all');
            if (result.success && result.data) {
                this.tasks = result.data;
            } else {
                this.tasks = [];
                if (result.error) {
                    vscode.window.showErrorMessage(`Failed to load tasks: ${result.error}`);
                }
            }
        } catch (error) {
            this.tasks = [];
            vscode.window.showErrorMessage(`Error loading tasks: ${error}`);
        } finally {
            this.isLoading = false;
        }
    }

    /**
     * Create status group headers
     */
    private getStatusGroups(): TaskTreeItem[] {
        const statusGroups = [
            { label: 'Todo', status: 'todo' },
            { label: 'Active', status: 'active' },
            { label: 'Done', status: 'done' },
            { label: 'Archived', status: 'archived' }
        ];

        return statusGroups.map(group => {
            const count = this.tasks.filter(t => t.status === group.status).length;
            const label = `${group.label} (${count})`;

            return new TaskTreeItem(
                label,
                true,
                undefined,
                vscode.TreeItemCollapsibleState.Expanded
            );
        });
    }

    /**
     * Get tasks for a specific status, with filters and sorting applied
     */
    private getTasksForStatus(status: string): TaskTreeItem[] {
        // Filter by status
        let filteredTasks = this.tasks.filter(t => t.status === status);

        // Apply search filter if active
        if (this.searchFilter) {
            filteredTasks = filteredTasks.filter(t =>
                t.title.toLowerCase().includes(this.searchFilter)
            );
        }

        // Apply priority filter
        if (this.filters.priority !== 'all') {
            filteredTasks = filteredTasks.filter(t => t.priority === this.filters.priority);
        }

        // Apply assignee filter
        if (this.filters.assignee !== 'all') {
            filteredTasks = filteredTasks.filter(t => t.assignee === this.filters.assignee);
        }

        // Apply tags filter (ANY match)
        if (this.filters.tags.length > 0) {
            filteredTasks = filteredTasks.filter(t => {
                if (!t.tags) return false;
                return this.filters.tags.some(tag => t.tags?.includes(tag));
            });
        }

        // Apply sorting
        filteredTasks = this.sortTasks(filteredTasks);

        // Map to tree items
        return filteredTasks.map(task => {
            const label = `[${task.priority.toUpperCase()}] ${task.title}`;
            return new TaskTreeItem(
                label,
                false,
                task,
                vscode.TreeItemCollapsibleState.None
            );
        });
    }

    /**
     * Sort tasks based on current sort options
     */
    private sortTasks(tasks: Task[]): Task[] {
        const sorted = [...tasks];

        sorted.sort((a, b) => {
            let comparison = 0;

            switch (this.sortOptions.by) {
                case 'priority':
                    const priorityOrder: Record<string, number> = {
                        'critical': 4,
                        'high': 3,
                        'medium': 2,
                        'low': 1
                    };
                    comparison = (priorityOrder[a.priority] || 2) - (priorityOrder[b.priority] || 2);
                    break;

                case 'created':
                    comparison = (a.created || '').localeCompare(b.created || '');
                    break;

                case 'updated':
                    comparison = (a.updated || '').localeCompare(b.updated || '');
                    break;

                case 'title':
                    comparison = a.title.localeCompare(b.title);
                    break;
            }

            return this.sortOptions.direction === 'asc' ? comparison : -comparison;
        });

        return sorted;
    }

    /**
     * Extract status from group label (e.g., "Todo (5)" -> "todo")
     */
    private getStatusFromLabel(label: string): string {
        const statusMap: Record<string, string> = {
            'Todo': 'todo',
            'Active': 'active',
            'Done': 'done',
            'Archived': 'archived'
        };

        // Extract base label before count
        const baseLabel = label.replace(/\s*\(\d+\)$/, '');
        return statusMap[baseLabel] || 'todo';
    }
}
