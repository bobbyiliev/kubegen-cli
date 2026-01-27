//! Build script for kubegen
//!
//! Embeds git commit hash and build date into the binary.
//! Validates all templates at compile time.

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get build date
    let build_date = chrono_lite_date();

    // Get Rust version
    let rust_version = get_rust_version();

    // Set environment variables for compilation
    println!("cargo:rustc-env=KUBEGEN_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=KUBEGEN_BUILD_DATE={}", build_date);
    println!("cargo:rustc-env=KUBEGEN_RUST_VERSION={}", rust_version);

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");

    // Validate templates
    println!("cargo:rerun-if-changed=templates/");
    validate_templates();
}

/// Get Rust version used for compilation
fn get_rust_version() -> String {
    Command::new("rustc")
        .args(["--version"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get current date in YYYY-MM-DD format without external dependencies
fn chrono_lite_date() -> String {
    Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Validate all templates in the templates/ directory
fn validate_templates() {
    let templates_dir = Path::new("templates");
    if !templates_dir.exists() {
        return;
    }

    let mut errors = Vec::new();
    validate_templates_in_dir(templates_dir, &mut errors);

    if !errors.is_empty() {
        for error in &errors {
            println!("cargo:warning=Template error: {}", error);
        }
        panic!("Template validation failed with {} error(s)", errors.len());
    }
}

fn validate_templates_in_dir(dir: &Path, errors: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(format!("Failed to read directory {:?}: {}", dir, e));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            validate_templates_in_dir(&path, errors);
        } else if path.extension().is_some_and(|ext| ext == "tmpl") {
            validate_template_file(&path, errors);
        }
    }
}

fn validate_template_file(path: &Path, errors: &mut Vec<String>) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!("{:?}: failed to read: {}", path, e));
            return;
        }
    };

    if let Err(e) = validate_template_syntax(&content) {
        errors.push(format!("{}: {}", path.display(), e));
    }
}

/// Validate template syntax (check for unclosed or empty variable tags)
fn validate_template_syntax(content: &str) -> Result<(), String> {
    let mut chars = content.chars().peekable();
    let mut line_num = 1;
    let mut col_num = 1;

    while let Some(c) = chars.next() {
        if c == '\n' {
            line_num += 1;
            col_num = 1;
            continue;
        }

        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second '{'
            let start_line = line_num;
            let start_col = col_num;

            let mut var_name = String::new();
            let mut found_end = false;

            while let Some(vc) = chars.next() {
                if vc == '\n' {
                    line_num += 1;
                    col_num = 1;
                } else {
                    col_num += 1;
                }

                if vc == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume second '}'
                    found_end = true;
                    break;
                }
                var_name.push(vc);
            }

            if !found_end {
                return Err(format!(
                    "line {}:{}: unclosed variable tag '{{{{{}...'",
                    start_line,
                    start_col,
                    var_name.chars().take(20).collect::<String>()
                ));
            }

            let trimmed = var_name.trim();
            if trimmed.is_empty() {
                return Err(format!(
                    "line {}:{}: empty variable name",
                    start_line, start_col
                ));
            }
        }

        col_num += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_template() {
        assert!(validate_template_syntax("Hello, {{name}}!").is_ok());
    }

    #[test]
    fn test_valid_multiple_variables() {
        assert!(validate_template_syntax("{{a}} and {{b}}").is_ok());
    }

    #[test]
    fn test_valid_no_variables() {
        assert!(validate_template_syntax("plain text").is_ok());
    }

    #[test]
    fn test_unclosed_variable() {
        let result = validate_template_syntax("Hello, {{name!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unclosed"));
    }

    #[test]
    fn test_empty_variable() {
        let result = validate_template_syntax("Hello, {{}}!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_whitespace_only_variable() {
        let result = validate_template_syntax("Hello, {{   }}!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_multiline_template() {
        let template = "line1: {{var1}}\nline2: {{var2}}\n";
        assert!(validate_template_syntax(template).is_ok());
    }

    #[test]
    fn test_unclosed_on_second_line() {
        let template = "line1: {{var1}}\nline2: {{var2";
        let result = validate_template_syntax(template);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("line 2"));
    }

    #[test]
    fn test_variable_with_spaces() {
        assert!(validate_template_syntax("{{ name }}").is_ok());
    }

    #[test]
    fn test_adjacent_variables() {
        assert!(validate_template_syntax("{{a}}{{b}}{{c}}").is_ok());
    }
}
