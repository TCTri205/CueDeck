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
| `extra_patterns` | `Vec<String>` | `[]` | Additional user-defined secret patterns. |

## 4. MCP Settings (`[mcp]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `search_limit` | `usize` | `10` | Max results for `read_context`. |

## 5. Author Settings (`[author]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `name` | `string` | `""` | Default author name for new cards. |
| `email` | `string` | `""` | Default author email (auto-filled from git if empty). |

> **Note**: Author settings are typically defined in the Global Config (`~/.config/cuedeck/config.toml`) and auto-filled during `cue card new`.

## 6. Watcher Settings (`[watcher]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | `bool` | `true` | Enable/disable file watching. |
| `debounce_ms` | `u64` | `500` | Milliseconds to wait before triggering rebuild after file change. |
| `ignore_patterns` | `Vec<String>` | `[".git/", ".cache/"]` | Patterns to exclude from watching. |

## 7. Cache Settings (`[cache]`)

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `cache_mode` | `string` | `"lazy"` | Cache strategy: `lazy`, `eager`, or `disabled`. |
| `memory_limit_mb` | `usize` | `512` | Maximum memory for in-memory cache (MB). |

---

## 7. Struct Definition (Rust)

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

#[derive(Debug, Deserialize, Clone)]
pub struct ParserConfig {
    pub ignore_patterns: Vec<String>,
    pub anchor_levels: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub secret_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpConfig {
    pub search_limit: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WatcherConfig {
    pub debounce_ms: u64,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthorConfig {
    pub name: String,
}
```

---
**Related Docs**: [MODULE_DESIGN.md](../02_architecture/MODULE_DESIGN.md), [KNOWLEDGE_BASE_STRUCTURE.md](./KNOWLEDGE_BASE_STRUCTURE.md)
