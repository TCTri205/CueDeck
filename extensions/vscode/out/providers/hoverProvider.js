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
exports.CuedeckHoverProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs/promises"));
class CuedeckHoverProvider {
    async provideHover(document, position, _token) {
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
        }
        catch (error) {
            return null;
        }
    }
}
exports.CuedeckHoverProvider = CuedeckHoverProvider;
//# sourceMappingURL=hoverProvider.js.map