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
exports.registerCommands = registerCommands;
const vscode = __importStar(require("vscode"));
const GraphPanel_1 = require("./panels/GraphPanel");
/**
 * Register all extension commands
 */
function registerCommands(context, client, taskTreeProvider // Will be passed from extension.ts
) {
    // Search command
    const searchCommand = vscode.commands.registerCommand('cuedeck.search', async () => {
        await searchDocuments(client, context);
    });
    // Graph view command
    const graphCommand = vscode.commands.registerCommand('cuedeck.graphView', () => {
        GraphPanel_1.GraphPanel.createOrShow(context.extensionUri, client);
    });
    // Refresh tasks command
    const refreshCommand = vscode.commands.registerCommand('cuedeck.refreshTasks', async () => {
        if (taskTreeProvider) {
            taskTreeProvider.refresh();
            vscode.window.showInformationMessage('Tasks refreshed!');
        }
    });
    // Search tasks command
    const searchTasksCommand = vscode.commands.registerCommand('cuedeck.searchTasks', async () => {
        if (taskTreeProvider) {
            const searchText = await vscode.window.showInputBox({
                prompt: 'Search tasks by title',
                placeHolder: 'Enter search text...',
                value: ''
            });
            if (searchText !== undefined) {
                taskTreeProvider.setSearchFilter(searchText);
                if (searchText) {
                    vscode.window.showInformationMessage(`Filtering tasks: "${searchText}"`);
                }
            }
        }
    });
    // Open task file command
    const openTaskCommand = vscode.commands.registerCommand('cuedeck.openTask', async (item) => {
        if (item.task) {
            try {
                const uri = vscode.Uri.file(item.task.file);
                const document = await vscode.workspace.openTextDocument(uri);
                await vscode.window.showTextDocument(document);
            }
            catch (error) {
                vscode.window.showErrorMessage(`Failed to open task: ${error}`);
            }
        }
    });
    // Change task status command
    const changeStatusCommand = vscode.commands.registerCommand('cuedeck.changeTaskStatus', async (item) => {
        if (!item.task) {
            return;
        }
        const newStatus = await vscode.window.showQuickPick(['todo', 'active', 'done', 'archived'], {
            placeHolder: `Current status: ${item.task.status}`,
            title: 'Select new status'
        });
        if (newStatus) {
            const result = await client.updateTask(item.task.id, { status: newStatus });
            if (result.success) {
                vscode.window.showInformationMessage(`Task status updated to: ${newStatus}`);
                if (taskTreeProvider) {
                    taskTreeProvider.refresh();
                }
            }
            else {
                vscode.window.showErrorMessage(`Failed to update status: ${result.error}`);
            }
        }
    });
    // Change task priority command
    const changePriorityCommand = vscode.commands.registerCommand('cuedeck.changeTaskPriority', async (item) => {
        if (!item.task) {
            return;
        }
        const newPriority = await vscode.window.showQuickPick(['low', 'medium', 'high', 'critical'], {
            placeHolder: `Current priority: ${item.task.priority}`,
            title: 'Select new priority'
        });
        if (newPriority) {
            const result = await client.updateTask(item.task.id, { priority: newPriority });
            if (result.success) {
                vscode.window.showInformationMessage(`Task priority updated to: ${newPriority}`);
                if (taskTreeProvider) {
                    taskTreeProvider.refresh();
                }
            }
            else {
                vscode.window.showErrorMessage(`Failed to update priority: ${result.error}`);
            }
        }
    });
    // Change task assignee command
    const changeAssigneeCommand = vscode.commands.registerCommand('cuedeck.changeTaskAssignee', async (item) => {
        if (!item.task) {
            return;
        }
        const newAssignee = await vscode.window.showInputBox({
            prompt: 'Enter assignee name (with or without @)',
            value: item.task.assignee || '',
            placeHolder: '@username'
        });
        if (newAssignee !== undefined) {
            // Remove @ prefix if present
            const cleanAssignee = newAssignee.startsWith('@')
                ? newAssignee.substring(1)
                : newAssignee;
            const result = await client.updateTask(item.task.id, { assignee: cleanAssignee });
            if (result.success) {
                vscode.window.showInformationMessage(`Task assignee updated to: @${cleanAssignee}`);
                if (taskTreeProvider) {
                    taskTreeProvider.refresh();
                }
            }
            else {
                vscode.window.showErrorMessage(`Failed to update assignee: ${result.error}`);
            }
        }
    });
    // Create task command
    const createTaskCommand = vscode.commands.registerCommand('cuedeck.createTask', async () => {
        const title = await vscode.window.showInputBox({
            prompt: 'Enter task title',
            placeHolder: 'Implement feature X'
        });
        if (!title) {
            return;
        }
        const result = await client.createTask(title);
        if (result.success && result.data) {
            vscode.window.showInformationMessage(`Created task: ${result.data.id}`);
            // Open the newly created file
            try {
                const uri = vscode.Uri.file(result.data.file);
                const document = await vscode.workspace.openTextDocument(uri);
                await vscode.window.showTextDocument(document);
            }
            catch (error) {
                vscode.window.showWarningMessage(`Task created but failed to open file: ${error}`);
            }
            if (taskTreeProvider) {
                taskTreeProvider.refresh();
            }
        }
        else {
            vscode.window.showErrorMessage(`Failed to create task: ${result.error}`);
        }
    });
    // Create task from template command
    const createFromTemplateCommand = vscode.commands.registerCommand('cuedeck.createTaskFromTemplate', async () => {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }
        // Get templates directory
        const templatesPath = vscode.Uri.joinPath(workspaceFolder.uri, '.cuedeck', 'templates');
        try {
            // List available templates
            const templates = await vscode.workspace.fs.readDirectory(templatesPath);
            const templateFiles = templates
                .filter(([name, type]) => type === vscode.FileType.File && name.endsWith('.md'))
                .map(([name]) => name);
            if (templateFiles.length === 0) {
                vscode.window.showErrorMessage('No templates found in .cuedeck/templates/');
                return;
            }
            // Show template picker
            const selectedTemplate = await vscode.window.showQuickPick(templateFiles.map(name => ({
                label: name.replace('.md', ''),
                description: `Template: ${name}`,
                value: name
            })), {
                title: 'Select Task Template',
                placeHolder: 'Choose a template...'
            });
            if (!selectedTemplate)
                return;
            // Prompt for task title
            const title = await vscode.window.showInputBox({
                prompt: 'Enter task title',
                placeHolder: 'e.g., Fix login button CSS',
                validateInput: (value) => {
                    return value.length === 0 ? 'Title cannot be empty' : undefined;
                }
            });
            if (!title)
                return;
            // Read template file
            const templateUri = vscode.Uri.joinPath(templatesPath, selectedTemplate.value);
            const templateContent = await vscode.workspace.fs.readFile(templateUri);
            let content = Buffer.from(templateContent).toString('utf8');
            // Variable substitution
            const now = new Date();
            const dateStr = now.toISOString().split('T')[0]; // YYYY-MM-DD
            const user = process.env.USERNAME || process.env.USER || 'user';
            const id = Math.random().toString(36).substring(2, 8); // 6-char random ID
            content = content.replace(/\{\{date\}\}/g, dateStr);
            content = content.replace(/\{\{user\}\}/g, user);
            content = content.replace(/\{\{title\}\}/g, title);
            content = content.replace(/\{\{id\}\}/g, id);
            // Create task using CLI
            const result = await client.createTask(title);
            if (result.success && result.data) {
                // Write template content to the created file
                const taskUri = vscode.Uri.file(result.data.file);
                await vscode.workspace.fs.writeFile(taskUri, Buffer.from(content, 'utf8'));
                vscode.window.showInformationMessage(`Created task from template: ${selectedTemplate.label}`);
                // Open the file
                const document = await vscode.workspace.openTextDocument(taskUri);
                await vscode.window.showTextDocument(document);
                // Refresh task tree
                if (taskTreeProvider) {
                    taskTreeProvider.refresh();
                }
            }
            else {
                vscode.window.showErrorMessage(`Failed to create task: ${result.error}`);
            }
        }
        catch (error) {
            vscode.window.showErrorMessage(`Error loading templates: ${error}`);
        }
    });
    // Filter tasks command
    const filterTasksCommand = vscode.commands.registerCommand('cuedeck.filterTasks', async () => {
        if (!taskTreeProvider)
            return;
        // Step 1: Select filter type
        const filterType = await vscode.window.showQuickPick([
            { label: '🎯 Priority', description: 'Filter by task priority', value: 'priority' },
            { label: '👤 Assignee', description: 'Filter by assignee', value: 'assignee' },
            { label: '🏷️  Tags', description: 'Filter by tags', value: 'tags' },
            { label: '🗑️  Clear All Filters', description: 'Remove all active filters', value: 'clear' }
        ], {
            title: 'Filter Tasks',
            placeHolder: 'Select filter type...'
        });
        if (!filterType)
            return;
        if (filterType.value === 'clear') {
            taskTreeProvider.clearFilters();
            vscode.window.showInformationMessage('All filters cleared');
            return;
        }
        // Step 2: Select filter value
        switch (filterType.value) {
            case 'priority':
                const priority = await vscode.window.showQuickPick([
                    { label: 'All Priorities', value: 'all' },
                    { label: '🔴 Critical', value: 'critical' },
                    { label: '🟠 High', value: 'high' },
                    { label: '🟡 Medium', value: 'medium' },
                    { label: '🟢 Low', value: 'low' }
                ], {
                    title: 'Select Priority',
                    placeHolder: 'Choose priority level...'
                });
                if (priority) {
                    taskTreeProvider.setFilters({ priority: priority.value });
                    vscode.window.showInformationMessage(`Filtered by: ${priority.label}`);
                }
                break;
            case 'assignee':
                // Get unique assignees from tasks
                const assignees = await getUniqueAssignees(client);
                const assigneeItems = [
                    { label: 'All Assignees', value: 'all' },
                    ...assignees.map(a => ({ label: `@${a}`, value: a }))
                ];
                const assignee = await vscode.window.showQuickPick(assigneeItems, {
                    title: 'Select Assignee',
                    placeHolder: 'Choose assignee...'
                });
                if (assignee) {
                    taskTreeProvider.setFilters({ assignee: assignee.value });
                    vscode.window.showInformationMessage(`Filtered by: ${assignee.label}`);
                }
                break;
            case 'tags':
                const allTags = await getUniqueTags(client);
                const selectedTags = await vscode.window.showQuickPick(allTags.map(tag => ({ label: `#${tag}`, value: tag })), {
                    canPickMany: true,
                    title: 'Select Tags',
                    placeHolder: 'Choose one or more tags...'
                });
                if (selectedTags) {
                    taskTreeProvider.setFilters({
                        tags: selectedTags.map(t => t.value)
                    });
                    vscode.window.showInformationMessage(`Filtered by tags: ${selectedTags.map(t => t.label).join(', ')}`);
                }
                break;
        }
    });
    // Sort tasks command
    const sortTasksCommand = vscode.commands.registerCommand('cuedeck.sortTasks', async () => {
        if (!taskTreeProvider)
            return;
        const sortBy = await vscode.window.showQuickPick([
            { label: '🎯 Priority (High to Low)', value: { by: 'priority', direction: 'desc' } },
            { label: '📅 Created Date (Newest First)', value: { by: 'created', direction: 'desc' } },
            { label: '📅 Created Date (Oldest First)', value: { by: 'created', direction: 'asc' } },
            { label: '🔄 Updated Date (Most Recent)', value: { by: 'updated', direction: 'desc' } },
            { label: '🔄 Updated Date (Least Recent)', value: { by: 'updated', direction: 'asc' } },
            { label: '📝 Title (A to Z)', value: { by: 'title', direction: 'asc' } },
            { label: '📝 Title (Z to A)', value: { by: 'title', direction: 'desc' } }
        ], {
            title: 'Sort Tasks',
            placeHolder: 'Select sort order...'
        });
        if (sortBy) {
            taskTreeProvider.setSortOptions(sortBy.value);
            vscode.window.showInformationMessage(`Sorted by: ${sortBy.label}`);
        }
    });
    context.subscriptions.push(searchCommand, graphCommand, refreshCommand, searchTasksCommand, openTaskCommand, changeStatusCommand, changePriorityCommand, changeAssigneeCommand, createTaskCommand, createFromTemplateCommand, filterTasksCommand, sortTasksCommand);
}
// Helper function to get unique assignees
async function getUniqueAssignees(client) {
    const result = await client.listTasks('all');
    if (!result.success || !result.data)
        return [];
    const assignees = new Set();
    result.data.forEach(task => {
        if (task.assignee)
            assignees.add(task.assignee);
    });
    return Array.from(assignees).sort();
}
// Helper function to get unique tags
async function getUniqueTags(client) {
    const result = await client.listTasks('all');
    if (!result.success || !result.data)
        return [];
    const tags = new Set();
    result.data.forEach(task => {
        if (task.tags) {
            task.tags.forEach(tag => tags.add(tag));
        }
    });
    return Array.from(tags).sort();
}
/**
 * Quick search panel implementation with recent searches
 */
async function searchDocuments(client, context) {
    // Get recent searches from workspace state
    const recentSearches = context?.workspaceState.get('recentSearches', []) || [];
    // Build search query options 
    const queryItems = [];
    // Add recent searches if any
    if (recentSearches.length > 0) {
        queryItems.push({ label: '$(clock) Recent Searches', kind: vscode.QuickPickItemKind.Separator });
        recentSearches.slice(0, 10).forEach((search, index) => {
            queryItems.push({
                label: `$(history) ${search}`,
                description: `Recent #${index + 1}`,
                detail: 'Click to search again'
            });
        });
        queryItems.push({ label: '', kind: vscode.QuickPickItemKind.Separator });
        queryItems.push({
            label: '$(trash) Clear Recent Searches',
            description: 'Remove all history'
        });
        queryItems.push({ label: '', kind: vscode.QuickPickItemKind.Separator });
    }
    // Add new search option
    queryItems.push({
        label: '$(search) New Search',
        description: 'Enter a new query'
    });
    // Show recent searches or go directly to input
    let query;
    if (recentSearches.length > 0) {
        const selectedItem = await vscode.window.showQuickPick(queryItems, {
            title: 'Search Documents',
            placeHolder: 'Select a recent search or start a new one...'
        });
        if (!selectedItem)
            return;
        if (selectedItem.label.includes('Clear Recent')) {
            await context?.workspaceState.update('recentSearches', []);
            vscode.window.showInformationMessage('Recent searches cleared');
            return;
        }
        if (selectedItem.label.includes('$(history)')) {
            // Extract query from label
            query = selectedItem.label.replace('$(history) ', '');
        }
    }
    // If no query selected from recent, prompt for new query
    if (!query) {
        query = await vscode.window.showInputBox({
            prompt: 'Search documents in workspace',
            placeHolder: 'Enter search query...',
            validateInput: (value) => {
                return value.length === 0 ? 'Query cannot be empty' : undefined;
            },
        });
        if (!query)
            return;
    }
    // First, ask for search mode
    const modeItems = [
        { label: 'Hybrid', description: 'Balance keyword and semantic search', detail: 'Default mode' },
        { label: 'Semantic', description: 'AI-powered meaning-based search', detail: 'Best for conceptual queries' },
        { label: 'Keyword', description: 'Exact text matching', detail: 'Best for precise terms' }
    ];
    const selectedMode = await vscode.window.showQuickPick(modeItems, {
        title: 'Select Search Mode',
        placeHolder: 'Choose how to search...'
    });
    if (!selectedMode) {
        return; // User cancelled
    }
    const mode = selectedMode.label.toLowerCase();
    // Save query to recent searches (at the beginning of array)
    if (context && query) {
        const updated = [query, ...recentSearches.filter(s => s !== query)].slice(0, 10);
        await context.workspaceState.update('recentSearches', updated);
    }
    // Show progress indicator
    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: `Searching for "${query}" (${selectedMode.label} mode)...`,
        cancellable: false,
    }, async () => {
        const result = await client.search(query, mode);
        if (!result.success || !result.data) {
            vscode.window.showErrorMessage(`Search failed: ${result.error || 'Unknown error'}`);
            return;
        }
        const searchResults = result.data;
        if (searchResults.length === 0) {
            vscode.window.showInformationMessage(`No results found for "${query}"`);
            return;
        }
        // Fetch previews for top 10 results
        const top10 = searchResults.slice(0, 10);
        const itemsWithPreview = await Promise.all(top10.map(async (item) => {
            const preview = await getFilePreview(item.path, query);
            return {
                label: `$(file) ${item.preview}`,
                description: `Score: ${item.score}`,
                detail: preview || item.path,
                path: item.path
            };
        }));
        // Remaining items without detailed preview
        const remaining = searchResults.slice(10).map((item) => ({
            label: `$(file) ${item.preview}`,
            description: `Score: ${item.score}`,
            detail: item.path,
            path: item.path
        }));
        const items = [...itemsWithPreview, ...remaining];
        // Show QuickPick
        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: `Found ${searchResults.length} results`,
            matchOnDescription: true,
            matchOnDetail: true,
        });
        if (selected && selected.path) {
            // Open selected file
            const uri = vscode.Uri.file(selected.path);
            const document = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(document);
        }
    });
}
/**
 * Get file preview for search results
 */
async function getFilePreview(filePath, query) {
    try {
        const uri = vscode.Uri.file(filePath);
        const document = await vscode.workspace.openTextDocument(uri);
        // Get first 3 non-empty lines
        const lines = document.getText().split('\n')
            .filter(line => line.trim().length > 0)
            .slice(0, 3);
        if (lines.length === 0)
            return null;
        let preview = lines.join(' | ');
        // Truncate if too long
        if (preview.length > 100) {
            preview = preview.substring(0, 97) + '...';
        }
        // Highlight query match (simple approach)
        if (query) {
            const lowerPreview = preview.toLowerCase();
            const lowerQuery = query.toLowerCase();
            const index = lowerPreview.indexOf(lowerQuery);
            if (index !== -1) {
                preview = preview.substring(0, index) +
                    '→ ' + preview.substring(index, index + query.length) + ' ←' +
                    preview.substring(index + query.length);
            }
        }
        return preview;
    }
    catch (error) {
        return null;
    }
}
//# sourceMappingURL=commands.js.map