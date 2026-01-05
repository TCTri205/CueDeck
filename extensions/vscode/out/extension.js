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
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const cuedeckClient_1 = require("./cuedeckClient");
const commands_1 = require("./commands");
const linkProvider_1 = require("./providers/linkProvider");
const hoverProvider_1 = require("./providers/hoverProvider");
const definitionProvider_1 = require("./providers/definitionProvider");
const TaskTreeProvider_1 = require("./providers/TaskTreeProvider");
const StatisticsProvider_1 = require("./providers/StatisticsProvider");
/**
 * Extension activation
 */
async function activate(context) {
    console.log('CueDeck extension is now active!');
    // Get configuration
    const config = vscode.workspace.getConfiguration('cuedeck');
    const cliPath = config.get('cliPath', 'cue');
    // Create client
    const client = new cuedeckClient_1.CuedeckClient(cliPath);
    // Check if CLI is available
    const isAvailable = await client.isAvailable();
    if (!isAvailable) {
        vscode.window.showWarningMessage(`CueDeck CLI not found at '${cliPath}'. Please install CueDeck or configure the path in settings.`);
    }
    else {
        vscode.window.showInformationMessage(`CueDeck extension loaded! Using CLI: ${cliPath}`);
    }
    // Listen for configuration changes
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('cuedeck.cliPath')) {
            const newPath = vscode.workspace.getConfiguration('cuedeck').get('cliPath', 'cue');
            client.updatePath(newPath);
            vscode.window.showInformationMessage(`CueDeck CLI path updated to: ${newPath}`);
        }
    }));
    // Register Task Tree Provider first (needed by commands)
    const taskTreeProvider = new TaskTreeProvider_1.TaskTreeProvider(client);
    context.subscriptions.push(vscode.window.registerTreeDataProvider('cuedeckTasks', taskTreeProvider));
    // Register Statistics Provider
    const statisticsProvider = new StatisticsProvider_1.StatisticsProvider(client);
    context.subscriptions.push(vscode.window.registerTreeDataProvider('cuedeckStats', statisticsProvider));
    // Setup file watcher for auto-refresh with debouncing
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        const workspaceFolder = vscode.workspace.workspaceFolders[0];
        // Use glob pattern to match both .cuedeck and cuedeck folders
        const pattern = new vscode.RelativePattern(workspaceFolder, '{.cuedeck,cuedeck}/cards/*.md');
        const watcher = vscode.workspace.createFileSystemWatcher(pattern);
        // Debounce refresh to avoid multiple rapid updates
        let refreshTimeout;
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
    context.subscriptions.push(vscode.workspace.onDidSaveTextDocument((document) => {
        // Check if saved file is a task card
        const filePath = document.uri.fsPath.replace(/\\/g, '/');
        if (filePath.includes('/cards/') && filePath.endsWith('.md')) {
            console.log('[CueDeck] Task file saved, refreshing:', filePath);
            setTimeout(() => {
                taskTreeProvider.refresh();
                statisticsProvider.refresh();
            }, 100);
        }
    }));
    // Register commands (pass taskTreeProvider for refresh capability)
    (0, commands_1.registerCommands)(context, client, taskTreeProvider);
    // Register status bar item
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = '$(graph) CueDeck';
    statusBarItem.command = 'cuedeck.search';
    statusBarItem.tooltip = 'Search CueDeck knowledge graph';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);
    // Register Link Provider
    context.subscriptions.push(vscode.languages.registerDocumentLinkProvider({ scheme: 'file', language: 'markdown' }, new linkProvider_1.CuedeckLinkProvider()));
    // Register Hover Provider
    context.subscriptions.push(vscode.languages.registerHoverProvider({ scheme: 'file', language: 'markdown' }, new hoverProvider_1.CuedeckHoverProvider()));
    // Register Definition Provider
    context.subscriptions.push(vscode.languages.registerDefinitionProvider({ scheme: 'file', language: 'markdown' }, new definitionProvider_1.CuedeckDefinitionProvider()));
}
/**
 * Extension deactivation
 */
function deactivate() {
    console.log('CueDeck extension is now deactivated');
}
//# sourceMappingURL=extension.js.map