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
exports.TaskTreeItem = void 0;
const vscode = __importStar(require("vscode"));
/**
 * Represents an item in the task tree view
 * Can be either a status group header or an individual task
 */
class TaskTreeItem extends vscode.TreeItem {
    constructor(label, isGroup = false, task, collapsibleState) {
        super(label, collapsibleState);
        this.isGroup = isGroup;
        this.task = task;
        if (isGroup) {
            // Group header styling
            this.contextValue = 'statusGroup';
            this.iconPath = this.getStatusIcon(label);
        }
        else if (task) {
            // Individual task styling
            this.contextValue = 'task';
            this.description = this.buildDescription(task);
            this.tooltip = this.buildTooltip(task);
            this.iconPath = this.getPriorityIcon(task.priority);
            // Make task clickable to open file
            this.command = {
                command: 'cuedeck.openTask',
                title: 'Open Task',
                arguments: [this]
            };
        }
    }
    /**
     * Build secondary text showing assignee and tags
     */
    buildDescription(task) {
        const parts = [];
        if (task.assignee) {
            parts.push(`@${task.assignee}`);
        }
        if (task.tags && task.tags.length > 0) {
            parts.push(task.tags.join(', '));
        }
        return parts.join(' • ');
    }
    /**
     * Build rich hover tooltip with task metadata
     */
    buildTooltip(task) {
        const tooltip = new vscode.MarkdownString();
        tooltip.isTrusted = true;
        tooltip.appendMarkdown(`**${task.title}**\n\n`);
        tooltip.appendMarkdown(`Status: \`${task.status}\` • Priority: \`${task.priority}\`\n\n`);
        if (task.assignee) {
            tooltip.appendMarkdown(`Assignee: @${task.assignee}\n\n`);
        }
        if (task.tags && task.tags.length > 0) {
            tooltip.appendMarkdown(`Tags: ${task.tags.map(t => `\`${t}\``).join(', ')}\n\n`);
        }
        if (task.created) {
            tooltip.appendMarkdown(`Created: ${task.created}\n\n`);
        }
        if (task.updated) {
            tooltip.appendMarkdown(`Updated: ${task.updated}\n\n`);
        }
        tooltip.appendMarkdown(`File: \`${task.file}\``);
        return tooltip;
    }
    /**
     * Get icon for status group headers
     */
    getStatusIcon(status) {
        const iconMap = {
            'Todo': 'checklist',
            'Active': 'sync',
            'Done': 'check-all',
            'Archived': 'archive'
        };
        return new vscode.ThemeIcon(iconMap[status] || 'file');
    }
    /**
     * Get icon for task priority
     */
    getPriorityIcon(priority) {
        const iconMap = {
            'critical': { icon: 'error', color: new vscode.ThemeColor('errorForeground') },
            'high': { icon: 'warning', color: new vscode.ThemeColor('editorWarning.foreground') },
            'medium': { icon: 'info' },
            'low': { icon: 'circle-outline' }
        };
        const config = iconMap[priority] || iconMap['medium'];
        return new vscode.ThemeIcon(config.icon, config.color);
    }
}
exports.TaskTreeItem = TaskTreeItem;
//# sourceMappingURL=TaskTreeItem.js.map