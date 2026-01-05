//! Week 4 Advanced Agent Tools - MCP handlers
//!
//! This module implements MCP tool handlers for code search and safe file editing.

use cue_common::Result;
use cue_core::{agent_fs, code_search};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::path::PathBuf;

/// Handle search_code MCP tool - search code using regex patterns
pub async fn handle_search_code(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct SearchCodeParams {
        pattern: String,
        file_glob: Option<String>,
        #[serde(default)]
        case_sensitive: bool,
        #[serde(default = "default_max_results")]
        max_results: usize,
    }

    fn default_max_results() -> usize {
        50
    }

    let params: SearchCodeParams = serde_json::from_value(params.unwrap_or_default())?;

    let workspace = env::var("CUE_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_default());

    // Perform code search
    let matches = code_search::search_code(
        &workspace,
        &params.pattern,
        params.file_glob.as_deref(),
        params.case_sensitive,
        params.max_results,
    )?;

    #[derive(Serialize)]
    struct SearchCodeResponse {
        matches: Vec<code_search::CodeMatch>,
        total_count: usize,
        truncated: bool,
    }

    let total_count = matches.len();
    let response = SearchCodeResponse {
        matches,
        total_count,
        truncated: total_count >= params.max_results,
    };

    serde_json::to_value(response).map_err(|e| e.into())
}

/// Handle read_file_lines MCP tool - read specific line range from file
pub async fn handle_read_file_lines(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct ReadFileLinesParams {
        path: String,
        start_line: usize,
        end_line: usize,
    }

    let params: ReadFileLinesParams = serde_json::from_value(params.unwrap_or_default())?;

    let workspace = env::var("CUE_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_default());

    // Read file lines
    let content = agent_fs::read_file_lines(
        &workspace,
        &params.path,
        params.start_line,
        params.end_line,
    )?;

    #[derive(Serialize)]
    struct ReadFileLinesResponse {
        path: String,
        start_line: usize,
        end_line: usize,
        content: String,
        line_count: usize,
    }

    let line_count = content.lines().count();
    let response = ReadFileLinesResponse {
        path: params.path,
        start_line: params.start_line,
        end_line: params.end_line,
        content,
        line_count,
    };

    serde_json::to_value(response).map_err(|e| e.into())
}

/// Handle replace_in_file MCP tool - find and replace text in file
pub async fn handle_replace_in_file(params: Option<Value>) -> Result<Value> {
    #[derive(Deserialize)]
    struct ReplaceInFileParams {
        path: String,
        find: String,
        replace: String,
        #[serde(default)]
        regex: bool,
    }

    let params: ReplaceInFileParams = serde_json::from_value(params.unwrap_or_default())?;

    let workspace = env::var("CUE_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_default());

    // Perform replacement
    let result = agent_fs::replace_in_file(
        &workspace,
        &params.path,
        &params.find,
        &params.replace,
        params.regex,
    )?;

    serde_json::to_value(result).map_err(|e| e.into())
}
