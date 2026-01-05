import * as vscode from 'vscode';
import * as path from 'path';


export class CuedeckDefinitionProvider implements vscode.DefinitionProvider {
    provideDefinition(
        document: vscode.TextDocument,
        position: vscode.Position,
        _token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Definition> {
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

        // Return location of the file
        // We use position 0,0 since we don't have line number info in @ref yet
        // In the future, we could support @ref(path:line)
        return new vscode.Location(vscode.Uri.file(absolutePath), new vscode.Position(0, 0));
    }
}
