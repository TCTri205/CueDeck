import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

export class CuedeckHoverProvider implements vscode.HoverProvider {
    async provideHover(
        document: vscode.TextDocument,
        position: vscode.Position,
        _token: vscode.CancellationToken
    ): Promise<vscode.Hover | null> {
        const range = document.getWordRangeAtPosition(position, /@ref\([^)]+\)/);
        if (!range) {
            return null;
        }

        const text = document.getText(range);
        const match = /@ref\(([^)]+)\)/.exec(text);
        if (!match) {
            return null;
        }

        let linkPath = match[1].trim();
        // Remove quotes if present
        if ((linkPath.startsWith('"') && linkPath.endsWith('"')) ||
            (linkPath.startsWith("'") && linkPath.endsWith("'"))) {
            linkPath = linkPath.substring(1, linkPath.length - 1);
        }

        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return null;
        }

        const absolutePath = path.join(workspaceFolder.uri.fsPath, linkPath);

        try {
            const content = await fs.readFile(absolutePath, 'utf-8');
            // Truncate content for preview (first 500 chars)
            const previewText = content.slice(0, 500) + (content.length > 500 ? '\n...' : '');

            const markdown = new vscode.MarkdownString();
            markdown.appendMarkdown(`**Preview: ${linkPath}**\n\n`);
            markdown.appendCodeblock(previewText, 'markdown');

            return new vscode.Hover(markdown, range);
        } catch (error) {
            return null;
        }
    }
}
