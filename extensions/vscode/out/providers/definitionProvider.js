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
exports.CuedeckDefinitionProvider = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
class CuedeckDefinitionProvider {
    provideDefinition(document, position, _token) {
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
exports.CuedeckDefinitionProvider = CuedeckDefinitionProvider;
//# sourceMappingURL=definitionProvider.js.map