# Configuration Reference

**File**: `.cuedeck/config.toml`  
**Env Prefix**: `CUEDECK_` (e.g., `CUEDECK_CORE__TOKEN_LIMIT=50000`)

## 1. Core Settings (`[core]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `token_limit` | `usize` | `32000` | Max tokens for `SCENE.md`. |
| `hash_algo` | `string` | `"sha256"` | Hashing algorithm. |

## 2. Parser Settings (`[parser]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `ignore_patterns` | `Vec<String>` | `["target/", ...]` | Glob patterns to skip. |
| `anchor_levels` | `Vec<u8>` | `[1, 2, 3]` | Header-levels (`#` to `###`) to extract. |

## 3. Security Settings (`[security]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `secret_patterns` | `Vec<String>` | `[sk-..., ghp_...]` | Regex list for masking. |

## 4. MCP Settings (`[mcp]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `search_limit` | `usize` | `10` | Max results for `read_context`. |

---

## 5. Struct Definition (Rust)

The corresponding Rust struct in `crates/cue_config/src/lib.rs`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub parser: ParserConfig,
    pub security: SecurityConfig,
    pub mcp: McpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoreConfig {
    pub token_limit: usize,
    pub hash_algo: String,
}

// ... (Mirroring TOML structure)
```

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [KNOWLEDGE_BASE_STRUCTURE.md](./KNOWLEDGE_BASE_STRUCTURE.md)
