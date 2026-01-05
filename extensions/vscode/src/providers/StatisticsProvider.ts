import * as vscode from 'vscode';
import { CuedeckClient } from '../cuedeckClient';
import { Task } from '../types';

/**
 * Tree item for statistics display
 */
class StatTreeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly value?: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState = vscode.TreeItemCollapsibleState.None
    ) {
        super(label, collapsibleState);

        if (value) {
            this.description = value;
        }

        // Set context value for different item types
        this.contextValue = 'stat';
    }
}

interface TaskStats {
    total: number;
    byStatus: {
        todo: number;
        active: number;
        done: number;
        archived: number;
    };
    byPriority: {
        critical: number;
        high: number;
        medium: number;
        low: number;
    };
    thisWeek: {
        created: number;
        updated: number;
    };
    completionRate: number;
}

/**
 * Tree data provider for task statistics
 */
export class StatisticsProvider implements vscode.TreeDataProvider<StatTreeItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<StatTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private stats: TaskStats | null = null;
    private tasks: Task[] = [];

    constructor(private client: CuedeckClient) { }

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: StatTreeItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: StatTreeItem): Promise<StatTreeItem[]> {
        if (!vscode.workspace.workspaceFolders) {
            return [];
        }

        // Load tasks and calculate stats
        await this.loadStats();

        if (!this.stats) {
            return [new StatTreeItem('No data available')];
        }

        if (!element) {
            // Root level - return main categories
            return this.getRootItems();
        }

        return [];
    }

    private async loadStats(): Promise<void> {
        const result = await this.client.listTasks('all');

        if (!result.success || !result.data) {
            this.stats = null;
            return;
        }

        this.tasks = result.data;
        this.stats = this.calculateStats(this.tasks);
    }

    private calculateStats(tasks: Task[]): TaskStats {
        const now = new Date();
        const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);

        const stats: TaskStats = {
            total: tasks.length,
            byStatus: {
                todo: 0,
                active: 0,
                done: 0,
                archived: 0
            },
            byPriority: {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0
            },
            thisWeek: {
                created: 0,
                updated: 0
            },
            completionRate: 0
        };

        tasks.forEach(task => {
            // Count by status
            if (task.status in stats.byStatus) {
                stats.byStatus[task.status as keyof typeof stats.byStatus]++;
            }

            // Count by priority
            if (task.priority in stats.byPriority) {
                stats.byPriority[task.priority as keyof typeof stats.byPriority]++;
            }

            // Count this week
            if (task.created) {
                const createdDate = new Date(task.created);
                if (createdDate >= weekAgo) {
                    stats.thisWeek.created++;
                }
            }

            if (task.updated) {
                const updatedDate = new Date(task.updated);
                if (updatedDate >= weekAgo) {
                    stats.thisWeek.updated++;
                }
            }
        });

        // Calculate completion rate
        const completable = stats.byStatus.todo + stats.byStatus.active + stats.byStatus.done;
        if (completable > 0) {
            stats.completionRate = Math.round((stats.byStatus.done / completable) * 100);
        }

        return stats;
    }

    private getRootItems(): StatTreeItem[] {
        if (!this.stats) return [];

        const items: StatTreeItem[] = [];

        // Total tasks
        items.push(new StatTreeItem('📊 Total Tasks', `${this.stats.total}`));

        // Blank separator
        items.push(new StatTreeItem(''));

        // Status breakdown
        items.push(new StatTreeItem('📋 By Status', '', vscode.TreeItemCollapsibleState.None));
        items.push(new StatTreeItem(
            `  Todo`,
            `${this.stats.byStatus.todo} ${this.getProgressBar(this.stats.byStatus.todo, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  Active`,
            `${this.stats.byStatus.active} ${this.getProgressBar(this.stats.byStatus.active, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  Done`,
            `${this.stats.byStatus.done} ${this.getProgressBar(this.stats.byStatus.done, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  Archived`,
            `${this.stats.byStatus.archived} ${this.getProgressBar(this.stats.byStatus.archived, this.stats.total)}`
        ));

        // Blank separator
        items.push(new StatTreeItem(''));

        // Priority breakdown
        items.push(new StatTreeItem('🎯 By Priority', '', vscode.TreeItemCollapsibleState.None));
        items.push(new StatTreeItem(
            `  🔴 Critical`,
            `${this.stats.byPriority.critical} ${this.getProgressBar(this.stats.byPriority.critical, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  🟠 High`,
            `${this.stats.byPriority.high} ${this.getProgressBar(this.stats.byPriority.high, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  🟡 Medium`,
            `${this.stats.byPriority.medium} ${this.getProgressBar(this.stats.byPriority.medium, this.stats.total)}`
        ));
        items.push(new StatTreeItem(
            `  🟢 Low`,
            `${this.stats.byPriority.low} ${this.getProgressBar(this.stats.byPriority.low, this.stats.total)}`
        ));

        // Blank separator
        items.push(new StatTreeItem(''));

        // This week stats
        items.push(new StatTreeItem('📅 This Week', '', vscode.TreeItemCollapsibleState.None));
        items.push(new StatTreeItem(`  Created`, `${this.stats.thisWeek.created} ${this.getTrendIcon(this.stats.thisWeek.created)}`));
        items.push(new StatTreeItem(`  Updated`, `${this.stats.thisWeek.updated} ${this.getTrendIcon(this.stats.thisWeek.updated)}`));

        // Blank separator
        items.push(new StatTreeItem(''));

        // Completion rate
        items.push(new StatTreeItem('✅ Completion Rate', `${this.stats.completionRate}% ${this.getCompletionBar(this.stats.completionRate)}`));

        return items;
    }

    /**
     * Generate ASCII progress bar
     */
    private getProgressBar(count: number, total: number): string {
        if (total === 0) return '';

        const percentage = count / total;
        const barLength = 10;
        const filled = Math.round(percentage * barLength);
        const empty = barLength - filled;

        return '█'.repeat(filled) + '░'.repeat(empty);
    }

    /**
     * Generate completion progress bar
     */
    private getCompletionBar(percentage: number): string {
        const barLength = 20;
        const filled = Math.round((percentage / 100) * barLength);
        const empty = barLength - filled;

        return '█'.repeat(filled) + '░'.repeat(empty);
    }

    /**
     * Get trend icon based on count
     */
    private getTrendIcon(count: number): string {
        if (count > 10) return '↑';
        if (count > 5) return '→';
        if (count > 0) return '↗';
        return '→';
    }
}
