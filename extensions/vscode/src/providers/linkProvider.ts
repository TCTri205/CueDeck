import * as vscode from 'vscode';
import * as path from 'path';


export class CuedeckLinkProvider implements vscode.DocumentLinkProvider {
    provideDocumentLinks(
        document: vscode.TextDocument,
        _token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.DocumentLink[]> {
        const links: vscode.DocumentLink[] = [];
        const text = document.getText();

        // Regex to match @ref(path)
        // Matches: @ref(path/to/file.md) or @ref("path/with spaces.md")
        const regex = /@ref\(([^)]+)\)/g;

        let match;
        while ((match = regex.exec(text)) !== null) {
            const startPos = document.positionAt(match.index);
            const endPos = document.positionAt(match.index + match[0].length);
            const range = new vscode.Range(startPos, endPos);

            // Extract path (remove quotes if present)
            let linkPath = match[1].trim();
            if ((linkPath.startsWith('"') && linkPath.endsWith('"')) ||
                (linkPath.startsWith("'") && linkPath.endsWith("'"))) {
                linkPath = linkPath.substring(1, linkPath.length - 1);
            }

            // Resolve absolute path
            const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
            if (workspaceFolder) {
                const absolutePath = path.join(workspaceFolder.uri.fsPath, linkPath);

                // Only create link if file exists (optional, but good UX)
                // For now, we create it always to allow creating new files via click
                const targetUri = vscode.Uri.file(absolutePath);

                const link = new vscode.DocumentLink(range, targetUri);
                link.tooltip = `Open ${linkPath}`;
                links.push(link);
            }
        }

        return links;
    }
}
