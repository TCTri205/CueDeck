import * as vscode from 'vscode';
import { Task } from '../types';

/**
 * Represents an item in the task tree view
 * Can be either a status group header or an individual task
 */
export class TaskTreeItem extends vscode.TreeItem {
    public readonly task?: Task;
    public readonly isGroup: boolean;

    constructor(
        label: string,
        isGroup: boolean = false,
        task?: Task,
        collapsibleState?: vscode.TreeItemCollapsibleState
    ) {
        super(label, collapsibleState);
        this.isGroup = isGroup;
        this.task = task;

        if (isGroup) {
            // Group header styling
            this.contextValue = 'statusGroup';
            this.iconPath = this.getStatusIcon(label);
        } else if (task) {
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
    private buildDescription(task: Task): string {
        const parts: string[] = [];
        
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
    private buildTooltip(task: Task): vscode.MarkdownString {
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
    private getStatusIcon(status: string): vscode.ThemeIcon {
        const iconMap: Record<string, string> = {
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
    private getPriorityIcon(priority: string): vscode.ThemeIcon {
        const iconMap: Record<string, { icon: string; color?: vscode.ThemeColor }> = {
            'critical': { icon: 'error', color: new vscode.ThemeColor('errorForeground') },
            'high': { icon: 'warning', color: new vscode.ThemeColor('editorWarning.foreground') },
            'medium': { icon: 'info' },
            'low': { icon: 'circle-outline' }
        };
        
        const config = iconMap[priority] || iconMap['medium'];
        return new vscode.ThemeIcon(config.icon, config.color);
    }
}
