import * as vscode from 'vscode';
import { CuedeckClient } from './cuedeckClient';
import { registerCommands } from './commands';
import { CuedeckLinkProvider } from './providers/linkProvider';
import { CuedeckHoverProvider } from './providers/hoverProvider';
import { CuedeckDefinitionProvider } from './providers/definitionProvider';
import { TaskTreeProvider } from './providers/TaskTreeProvider';
import { StatisticsProvider } from './providers/StatisticsProvider';

/**
 * Extension activation
 */
export async function activate(context: vscode.ExtensionContext) {
    console.log('CueDeck extension is now active!');

    // Get configuration
    const config = vscode.workspace.getConfiguration('cuedeck');
    const cliPath = config.get<string>('cliPath', 'cue');

    // Create client
    const client = new CuedeckClient(cliPath);

    // Check if CLI is available
    const isAvailable = await client.isAvailable();
    if (!isAvailable) {
        vscode.window.showWarningMessage(
            `CueDeck CLI not found at '${cliPath}'. Please install CueDeck or configure the path in settings.`
        );
    } else {
        vscode.window.showInformationMessage(`CueDeck extension loaded! Using CLI: ${cliPath}`);
    }

    // Listen for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('cuedeck.cliPath')) {
                const newPath = vscode.workspace.getConfiguration('cuedeck').get<string>('cliPath', 'cue');
                client.updatePath(newPath);
                vscode.window.showInformationMessage(`CueDeck CLI path updated to: ${newPath}`);
            }
        })
    );

    // Register Task Tree Provider first (needed by commands)
    const taskTreeProvider = new TaskTreeProvider(client);
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('cuedeckTasks', taskTreeProvider)
    );

    // Register Statistics Provider
    const statisticsProvider = new StatisticsProvider(client);
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('cuedeckStats', statisticsProvider)
    );

    // Setup file watcher for auto-refresh with debouncing
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        const workspaceFolder = vscode.workspace.workspaceFolders[0];

        // Use glob pattern to match both .cuedeck and cuedeck folders
        const pattern = new vscode.RelativePattern(
            workspaceFolder,
            '{.cuedeck,cuedeck}/cards/*.md'
        );

        const watcher = vscode.workspace.createFileSystemWatcher(pattern);

        // Debounce refresh to avoid multiple rapid updates
        let refreshTimeout: NodeJS.Timeout | undefined;
        const debouncedRefresh = () => {
            if (refreshTimeout) {
                clearTimeout(refreshTimeout);
            }
            refreshTimeout = setTimeout(() => {
                console.log('[CueDeck] Refreshing task tree and statistics');
                taskTreeProvider.refresh();
                statisticsProvider.refresh();
            }, 300); // 300ms debounce
        };

        watcher.onDidChange((uri) => {
            console.log('[CueDeck] File changed:', uri.fsPath);
            debouncedRefresh();
        });
        watcher.onDidCreate((uri) => {
            console.log('[CueDeck] File created:', uri.fsPath);
            debouncedRefresh();
        });
        watcher.onDidDelete((uri) => {
            console.log('[CueDeck] File deleted:', uri.fsPath);
            debouncedRefresh();
        });

        context.subscriptions.push(watcher);
        console.log('[CueDeck] File watcher registered for:', pattern.pattern);
    }

    // Fallback: Auto-refresh on document save (more reliable than file watcher)
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((document) => {
            // Check if saved file is a task card
            const filePath = document.uri.fsPath.replace(/\\/g, '/');
            if (filePath.includes('/cards/') && filePath.endsWith('.md')) {
                console.log('[CueDeck] Task file saved, refreshing:', filePath);
                setTimeout(() => {
                    taskTreeProvider.refresh();
                    statisticsProvider.refresh();
                }, 100);
            }
        })
    );

    // Register commands (pass taskTreeProvider for refresh capability)
    registerCommands(context, client, taskTreeProvider);

    // Register status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(graph) CueDeck';
    statusBarItem.command = 'cuedeck.search';
    statusBarItem.tooltip = 'Search CueDeck knowledge graph';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Register Link Provider
    context.subscriptions.push(
        vscode.languages.registerDocumentLinkProvider(
            { scheme: 'file', language: 'markdown' },
            new CuedeckLinkProvider()
        )
    );

    // Register Hover Provider
    context.subscriptions.push(
        vscode.languages.registerHoverProvider(
            { scheme: 'file', language: 'markdown' },
            new CuedeckHoverProvider()
        )
    );

    // Register Definition Provider
    context.subscriptions.push(
        vscode.languages.registerDefinitionProvider(
            { scheme: 'file', language: 'markdown' },
            new CuedeckDefinitionProvider()
        )
    );
}

/**
 * Extension deactivation
 */
export function deactivate() {
    console.log('CueDeck extension is now deactivated');
}
