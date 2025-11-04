//! Code Analyzer Plugin Example

use agent_mem_plugin_sdk::*;
use extism_pdk::*;

/// Analyze code
#[plugin_fn]
pub fn analyze_code(input: String) -> FnResult<String> {
    // Parse input
    let input: CodeInput = serde_json::from_str(&input)?;
    
    // Choose analyzer based on language
    let analysis = match input.language.as_str() {
        "rust" => analyze_rust_code(&input.code)?,
        "python" => analyze_python_code(&input.code)?,
        _ => {
            return Err(ExtismError::msg(format!(
                "Unsupported language: {}",
                input.language
            )));
        }
    };
    
    // Log analysis
    let _ = host::log(
        "info",
        &format!("Analyzed {} code with {} functions", input.language, analysis.functions.len()),
    );
    
    // Return result
    Ok(serde_json::to_string(&analysis)?)
}

/// Analyze Rust code
fn analyze_rust_code(code: &str) -> Result<CodeAnalysis, ExtismError> {
    let mut functions = Vec::new();
    let mut imports = Vec::new();
    let mut patterns = Vec::new();
    
    // Simple pattern matching (in production, use a proper parser)
    for (i, line) in code.lines().enumerate() {
        // Detect function definitions
        if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") {
            let name = extract_function_name(line);
            functions.push(Function {
                name,
                line_start: i + 1,
                line_end: i + 1,
                parameters: vec![],
            });
        }
        
        // Detect imports
        if line.trim().starts_with("use ") {
            imports.push(line.trim().to_string());
        }
        
        // Detect patterns
        if line.contains("unwrap()") {
            patterns.push(CodePattern {
                pattern_type: "error_handling".to_string(),
                description: "Using unwrap() - consider proper error handling".to_string(),
                location: format!("Line {}", i + 1),
            });
        }
    }
    
    Ok(CodeAnalysis {
        language: "rust".to_string(),
        functions,
        imports,
        patterns,
        complexity: calculate_complexity(code),
    })
}

/// Analyze Python code (simplified)
fn analyze_python_code(code: &str) -> Result<CodeAnalysis, ExtismError> {
    let mut functions = Vec::new();
    let mut imports = Vec::new();
    
    for (i, line) in code.lines().enumerate() {
        if line.trim().starts_with("def ") {
            let name = line
                .trim()
                .strip_prefix("def ")
                .and_then(|s| s.split('(').next())
                .unwrap_or("unknown")
                .to_string();
            functions.push(Function {
                name,
                line_start: i + 1,
                line_end: i + 1,
                parameters: vec![],
            });
        }
        
        if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
            imports.push(line.trim().to_string());
        }
    }
    
    Ok(CodeAnalysis {
        language: "python".to_string(),
        functions,
        imports,
        patterns: vec![],
        complexity: calculate_complexity(code),
    })
}

/// Extract function name from line
fn extract_function_name(line: &str) -> String {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.split('(').next())
        .unwrap_or("unknown")
        .to_string()
}

/// Calculate code complexity
fn calculate_complexity(code: &str) -> i32 {
    let mut complexity = 1;
    
    for line in code.lines() {
        let line = line.trim();
        if line.starts_with("if ") || line.starts_with("else if ") || line.starts_with("elif ") {
            complexity += 1;
        }
        if line.starts_with("match ") || line.contains("=> ") {
            complexity += 1;
        }
        if line.starts_with("for ") || line.starts_with("while ") {
            complexity += 1;
        }
    }
    
    complexity
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "code-analyzer",
        version: "0.1.0",
        description: "Code analyzer for Rust and Python",
        author: "AgentMem Team",
        plugin_type: PluginType::CodeAnalyzer,
        capabilities: [Capability::LoggingAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}

