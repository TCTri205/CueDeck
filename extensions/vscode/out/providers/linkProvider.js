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
exports.CuedeckLinkProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
class CuedeckLinkProvider {
    provideDocumentLinks(document, _token) {
        const links = [];
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
exports.CuedeckLinkProvider = CuedeckLinkProvider;
//# sourceMappingURL=linkProvider.js.map