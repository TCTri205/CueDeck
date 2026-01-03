import * as vscode from 'vscode';
import { CuedeckClient } from './cuedeckClient';

/**
 * Register all CueDeck commands
 */
export function registerCommands(
    context: vscode.ExtensionContext,
    client: CuedeckClient
): void {
    // Search command
    context.subscriptions.push(
        vscode.commands.registerCommand('cuedeck.search', async () => {
            const query = await vscode.window.showInputBox({
                prompt: 'Enter search query',
                placeHolder: 'e.g., authentication flow',
            });

            if (!query) {
                return;
            }

            const result = await client.search(query);

            if (!result.success || !result.data) {
                vscode.window.showErrorMessage(
                    `Search failed: ${result.error || 'Unknown error'}`
                );
                return;
            }

            // Show results in quick pick
            const items = result.data.map((item) => ({
                label: item.path,
                description: `Score: ${item.score.toFixed(2)}`,
                detail: item.preview,
                path: item.path,
            }));

            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: 'Select a document to open',
            });

            if (selected) {
                const doc = await vscode.workspace.openTextDocument(selected.path);
                await vscode.window.showTextDocument(doc);
            }
        })
    );

    // Graph view command
    context.subscriptions.push(
        vscode.commands.registerCommand('cuedeck.graphView', async () => {
            vscode.window.showInformationMessage(
                'Graph visualization coming in Phase 8.2!'
            );
        })
    );

    // Refresh tasks command
    context.subscriptions.push(
        vscode.commands.registerCommand('cuedeck.refreshTasks', async () => {
            vscode.window.showInformationMessage('Refreshing tasks...');
            // TODO: Implement task refresh
        })
    );
}
