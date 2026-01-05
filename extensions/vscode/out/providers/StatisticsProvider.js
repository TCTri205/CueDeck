"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.StatisticsProvider = void 0;
const vscode = __importStar(require("vscode"));
/**
 * Tree item for statistics display
 */
class StatTreeItem extends vscode.TreeItem {
    constructor(label, value, collapsibleState = vscode.TreeItemCollapsibleState.None) {
        super(label, collapsibleState);
        this.label = label;
        this.value = value;
        this.collapsibleState = collapsibleState;
        if (value) {
            this.description = value;
        }
        // Set context value for different item types
        this.contextValue = 'stat';
    }
}
/**
 * Tree data provider for task statistics
 */
class StatisticsProvider {
    constructor(client) {
        this.client = client;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        this.stats = null;
        this.tasks = [];
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    async getChildren(element) {
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
    async loadStats() {
        const result = await this.client.listTasks('all');
        if (!result.success || !result.data) {
            this.stats = null;
            return;
        }
        this.tasks = result.data;
        this.stats = this.calculateStats(this.tasks);
    }
    calculateStats(tasks) {
        const now = new Date();
        const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
        const stats = {
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
                stats.byStatus[task.status]++;
            }
            // Count by priority
            if (task.priority in stats.byPriority) {
                stats.byPriority[task.priority]++;
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
    getRootItems() {
        if (!this.stats)
            return [];
        const items = [];
        // Total tasks
        items.push(new StatTreeItem('📊 Total Tasks', `${this.stats.total}`));
        // Blank separator
        items.push(new StatTreeItem(''));
        // Status breakdown
        items.push(new StatTreeItem('📋 By Status', '', vscode.TreeItemCollapsibleState.None));
        items.push(new StatTreeItem(`  Todo`, `${this.stats.byStatus.todo} ${this.getProgressBar(this.stats.byStatus.todo, this.stats.total)}`));
        items.push(new StatTreeItem(`  Active`, `${this.stats.byStatus.active} ${this.getProgressBar(this.stats.byStatus.active, this.stats.total)}`));
        items.push(new StatTreeItem(`  Done`, `${this.stats.byStatus.done} ${this.getProgressBar(this.stats.byStatus.done, this.stats.total)}`));
        items.push(new StatTreeItem(`  Archived`, `${this.stats.byStatus.archived} ${this.getProgressBar(this.stats.byStatus.archived, this.stats.total)}`));
        // Blank separator
        items.push(new StatTreeItem(''));
        // Priority breakdown
        items.push(new StatTreeItem('🎯 By Priority', '', vscode.TreeItemCollapsibleState.None));
        items.push(new StatTreeItem(`  🔴 Critical`, `${this.stats.byPriority.critical} ${this.getProgressBar(this.stats.byPriority.critical, this.stats.total)}`));
        items.push(new StatTreeItem(`  🟠 High`, `${this.stats.byPriority.high} ${this.getProgressBar(this.stats.byPriority.high, this.stats.total)}`));
        items.push(new StatTreeItem(`  🟡 Medium`, `${this.stats.byPriority.medium} ${this.getProgressBar(this.stats.byPriority.medium, this.stats.total)}`));
        items.push(new StatTreeItem(`  🟢 Low`, `${this.stats.byPriority.low} ${this.getProgressBar(this.stats.byPriority.low, this.stats.total)}`));
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
    getProgressBar(count, total) {
        if (total === 0)
            return '';
        const percentage = count / total;
        const barLength = 10;
        const filled = Math.round(percentage * barLength);
        const empty = barLength - filled;
        return '█'.repeat(filled) + '░'.repeat(empty);
    }
    /**
     * Generate completion progress bar
     */
    getCompletionBar(percentage) {
        const barLength = 20;
        const filled = Math.round((percentage / 100) * barLength);
        const empty = barLength - filled;
        return '█'.repeat(filled) + '░'.repeat(empty);
    }
    /**
     * Get trend icon based on count
     */
    getTrendIcon(count) {
        if (count > 10)
            return '↑';
        if (count > 5)
            return '→';
        if (count > 0)
            return '↗';
        return '→';
    }
}
exports.StatisticsProvider = StatisticsProvider;
//# sourceMappingURL=StatisticsProvider.js.map