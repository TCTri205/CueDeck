import * as vscode from 'vscode';
import { CuedeckClient } from './cuedeckClient';
import { registerCommands } from './commands';

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
        vscode.window.showInformationMessage('CueDeck extension loaded successfully!');
    }

    // Register commands
    registerCommands(context, client);

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
}

/**
 * Extension deactivation
 */
export function deactivate() {
    console.log('CueDeck extension is now deactivated');
}
