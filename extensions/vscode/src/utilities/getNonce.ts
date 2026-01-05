/**
 * A helper function that returns a unique alphanumeric string (nonce).
 *
 * @remarks The nonce is primarily used with the Content Security Policy (CSP)
 * in the webview to whitelist specific scripts.
 *
 * @returns A nonce string
 */
export function getNonce() {
    let text = "";
    const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for (let i = 0; i < 32; i++) {
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
}
