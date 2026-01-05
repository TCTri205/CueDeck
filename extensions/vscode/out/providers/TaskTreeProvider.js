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
exports.TaskTreeProvider = void 0;
const vscode = __importStar(require("vscode"));
const TaskTreeItem_1 = require("./TaskTreeItem");
/**
 * Tree data provider for the CueDeck tasks sidebar
 * Displays tasks grouped by status (todo/active/done/archived)
 */
class TaskTreeProvider {
    constructor(client) {
        this.client = client;
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        this.tasks = [];
        this.isLoading = false;
        this.searchFilter = '';
        this.filters = {
            priority: 'all',
            assignee: 'all',
            tags: []
        };
        this.sortOptions = {
            by: 'priority',
            direction: 'desc'
        };
    }
    /**
     * Refresh the tree view
     */
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    /**
     * Set search filter and refresh view
     */
    setSearchFilter(filter) {
        this.searchFilter = filter.toLowerCase().trim();
        this.refresh();
    }
    /**
     * Set task filters and refresh view
     */
    setFilters(filters) {
        this.filters = { ...this.filters, ...filters };
        this.refresh();
    }
    /**
     * Clear all filters
     */
    clearFilters() {
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
    getActiveFilterCount() {
        let count = 0;
        if (this.filters.priority !== 'all')
            count++;
        if (this.filters.assignee !== 'all')
            count++;
        if (this.filters.tags.length > 0)
            count++;
        return count;
    }
    /**
     * Set sort options and refresh view
     */
    setSortOptions(options) {
        this.sortOptions = options;
        this.refresh();
    }
    /**
     * Get tree item for display
     */
    getTreeItem(element) {
        return element;
    }
    /**
     * Get children of a tree item
     * - If element is undefined, return status groups (root level)
     * - If element is a status group, return tasks in that status
     */
    async getChildren(element) {
        // Check if workspace is available
        if (!vscode.workspace.workspaceFolders) {
            return [];
        }
        if (!element) {
            // Root level: return status groups
            await this.loadTasks();
            return this.getStatusGroups();
        }
        else if (element.isGroup) {
            // Status group: return tasks in that status
            const status = this.getStatusFromLabel(element.label);
            return this.getTasksForStatus(status);
        }
        return [];
    }
    /**
     * Load all tasks from CLI
     */
    async loadTasks() {
        if (this.isLoading) {
            return;
        }
        this.isLoading = true;
        try {
            const result = await this.client.listTasks('all');
            if (result.success && result.data) {
                this.tasks = result.data;
            }
            else {
                this.tasks = [];
                if (result.error) {
                    vscode.window.showErrorMessage(`Failed to load tasks: ${result.error}`);
                }
            }
        }
        catch (error) {
            this.tasks = [];
            vscode.window.showErrorMessage(`Error loading tasks: ${error}`);
        }
        finally {
            this.isLoading = false;
        }
    }
    /**
     * Create status group headers
     */
    getStatusGroups() {
        const statusGroups = [
            { label: 'Todo', status: 'todo' },
            { label: 'Active', status: 'active' },
            { label: 'Done', status: 'done' },
            { label: 'Archived', status: 'archived' }
        ];
        return statusGroups.map(group => {
            const count = this.tasks.filter(t => t.status === group.status).length;
            const label = `${group.label} (${count})`;
            return new TaskTreeItem_1.TaskTreeItem(label, true, undefined, vscode.TreeItemCollapsibleState.Expanded);
        });
    }
    /**
     * Get tasks for a specific status, with filters and sorting applied
     */
    getTasksForStatus(status) {
        // Filter by status
        let filteredTasks = this.tasks.filter(t => t.status === status);
        // Apply search filter if active
        if (this.searchFilter) {
            filteredTasks = filteredTasks.filter(t => t.title.toLowerCase().includes(this.searchFilter));
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
                if (!t.tags)
                    return false;
                return this.filters.tags.some(tag => t.tags?.includes(tag));
            });
        }
        // Apply sorting
        filteredTasks = this.sortTasks(filteredTasks);
        // Map to tree items
        return filteredTasks.map(task => {
            const label = `[${task.priority.toUpperCase()}] ${task.title}`;
            return new TaskTreeItem_1.TaskTreeItem(label, false, task, vscode.TreeItemCollapsibleState.None);
        });
    }
    /**
     * Sort tasks based on current sort options
     */
    sortTasks(tasks) {
        const sorted = [...tasks];
        sorted.sort((a, b) => {
            let comparison = 0;
            switch (this.sortOptions.by) {
                case 'priority':
                    const priorityOrder = {
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
    getStatusFromLabel(label) {
        const statusMap = {
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
exports.TaskTreeProvider = TaskTreeProvider;
//# sourceMappingURL=TaskTreeProvider.js.map