"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getNonce = getNonce;
/**
 * A helper function that returns a unique alphanumeric string (nonce).
 *
 * @remarks The nonce is primarily used with the Content Security Policy (CSP)
 * in the webview to whitelist specific scripts.
 *
 * @returns A nonce string
 */
function getNonce() {
    let text = "";
    const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for (let i = 0; i < 32; i++) {
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
}
//# sourceMappingURL=getNonce.js.map