import * as vscode from 'vscode';
import { getUri } from '../utilities/getUri';
import { getNonce } from '../utilities/getNonce';
import { CuedeckClient } from '../cuedeckClient';

/**
 * Manages the Graph Visualization WebView
 */
export class GraphPanel {
    public static currentPanel: GraphPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private readonly _client: CuedeckClient;
    private readonly _disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri, client: CuedeckClient) {
        this._panel = panel;
        this._extensionUri = extensionUri;
        this._client = client;

        // Set the webview's initial html content
        this._update();

        // Listen for when the panel is disposed
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Update the content based on view state changes
        this._panel.onDidChangeViewState(
            _e => {
                if (this._panel.visible) {
                    this._update();
                }
            },
            null,
            this._disposables
        );

        // Handle messages from the webview
        this._panel.webview.onDidReceiveMessage(
            async message => {
                switch (message.command) {
                    case 'alert':
                        vscode.window.showErrorMessage(message.text);
                        return;
                    case 'openFile':
                        this._openFile(message.path);
                        return;
                    case 'refresh':
                        await this._refreshGraph();
                        return;
                }
            },
            null,
            this._disposables
        );
    }

    /**
     * Create or show the Graph Panel
     */
    public static createOrShow(extensionUri: vscode.Uri, client: CuedeckClient) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        // If we already have a panel, show it.
        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel._panel.reveal(column);
            return;
        }

        // Otherwise, create a new panel.
        const panel = vscode.window.createWebviewPanel(
            'cuedeck.graphPanel',
            'CueDeck: Knowledge Graph',
            column || vscode.ViewColumn.One,
            {
                // Enable javascript in the webview
                enableScripts: true,
                // And restrict the webview to only loading content from our extension's `media` directory.
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'out'),
                    vscode.Uri.joinPath(extensionUri, 'media')
                ]
            }
        );

        GraphPanel.currentPanel = new GraphPanel(panel, extensionUri, client);
    }

    /**
     * Dispose of the panel
     */
    public dispose() {
        GraphPanel.currentPanel = undefined;

        // Clean up our resources
        this._panel.dispose();

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    /**
     * Open a file in the editor
     */
    private async _openFile(filePath: string) {
        try {
            const uri = vscode.Uri.file(filePath);
            const doc = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(doc);
        } catch (e) {
            vscode.window.showErrorMessage(`Could not open file: ${filePath}`);
        }
    }

    /**
     * Refresh graph data
     */
    private async _refreshGraph() {
        const result = await this._client.exportGraph();
        if (result.success && result.data) {
            this._panel.webview.postMessage({ command: 'graphData', data: result.data });
        } else {
            vscode.window.showErrorMessage(`Failed to load graph: ${result.error}`);
            this._panel.webview.postMessage({ command: 'error', error: result.error });
        }
    }

    /**
     * Update the Webview Content
     */
    private _update() {
        const webview = this._panel.webview;
        this._panel.webview.html = this._getHtmlForWebview(webview);
    }

    /**
     * Get HTML for webview
     */
    private _getHtmlForWebview(webview: vscode.Webview) {
        // Use a nonce to only allow specific scripts to be run
        const nonce = getNonce();

        // Get the URI for scripts and css
        const cytoscapeUri = getUri(webview, this._extensionUri, ["media", "cytoscape.min.js"]);
        const scriptUri = getUri(webview, this._extensionUri, ["media", "main.js"]);
        const styleUri = getUri(webview, this._extensionUri, ["media", "main.css"]);

        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource}; script-src 'nonce-${nonce}';">
                <link href="${styleUri}" rel="stylesheet">
                <title>CueDeck Graph</title>
            </head>
            <body>
                <div class="controls">
                    <button id="refresh-btn">Refresh Graph</button>
                    <button id="fit-btn">Fit to Screen</button>
                    <button id="layout-btn">Re-layout</button>
                    <select id="layout-selector">
                        <option value="cose">Force-Directed (Cose)</option>
                        <option value="dagre">Hierarchical (Dagre)</option>
                        <option value="circle">Circle</option>
                        <option value="grid">Grid</option>
                        <option value="breadthfirst">Breadthfirst</option>
                    </select>
                    <span style="flex-grow:1"></span>
                    <span id="status">Ready - Click Refresh to load graph</span>
                </div>
                <div id="graph-container">
                    <div id="cy"></div>
                    <div class="legend">
                        <div class="legend-section">
                            <h4>Status</h4>
                            <div class="legend-item">
                                <span class="dot" style="background:#3794ff"></span> Todo
                            </div>
                            <div class="legend-item">
                                <span class="dot" style="background:#ff9500"></span> Active
                            </div>
                            <div class="legend-item">
                                <span class="dot" style="background:#28a745"></span> Done
                            </div>
                            <div class="legend-item">
                                <span class="dot" style="background:#6c757d"></span> Archived
                            </div>
                        </div>
                        <div class="legend-section">
                            <h4>Priority</h4>
                            <div class="legend-item">🔴 Critical</div>
                            <div class="legend-item">🟠 High</div>
                            <div class="legend-item">🟢 Low</div>
                        </div>
                    </div>
                </div>
                <script nonce="${nonce}" src="${cytoscapeUri}"></script>
                <script nonce="${nonce}" src="${scriptUri}"></script>
            </body>
            </html>`;
    }
}
