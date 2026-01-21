//! Interactive prompts for kubegen
//!
//! Provides user-friendly prompts for gathering input when CLI flags are missing.
//! Supports non-interactive mode for CI/CD and scripting scenarios.

use dialoguer::{Confirm, Input, Select};
use std::io::{self, IsTerminal};

use crate::error::{KubegenError, Result};

/// Check if we're running in an interactive terminal
pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Prompt for a required string value
///
/// # Arguments
/// * `prompt` - The prompt message to display
/// * `default` - Optional default value
/// * `non_interactive` - If true, returns default or error
///
/// # Returns
/// The user's input or the default value
pub fn prompt_string(prompt: &str, default: Option<&str>, non_interactive: bool) -> Result<String> {
    if non_interactive || !is_interactive() {
        return default.map(|s| s.to_string()).ok_or_else(|| {
            KubegenError::ValidationError(format!(
                "Missing required value for '{}' in non-interactive mode",
                prompt
            ))
        });
    }

    let mut input = Input::<String>::new().with_prompt(prompt);
    if let Some(def) = default {
        input = input.default(def.to_string());
    }

    input.interact_text().map_err(|e| {
        KubegenError::ValidationError(format!("Failed to read input for '{}': {}", prompt, e))
    })
}

/// Prompt for confirmation (yes/no)
///
/// # Arguments
/// * `prompt` - The prompt message to display
/// * `default` - Default value if user just presses Enter
/// * `non_interactive` - If true, returns the default value
///
/// # Returns
/// true for yes, false for no
pub fn prompt_confirm(prompt: &str, default: bool, non_interactive: bool) -> Result<bool> {
    if non_interactive || !is_interactive() {
        return Ok(default);
    }

    Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|e| {
            KubegenError::ValidationError(format!(
                "Failed to read confirmation for '{}': {}",
                prompt, e
            ))
        })
}

/// Prompt user to select from a list of options
///
/// # Arguments
/// * `prompt` - The prompt message to display
/// * `options` - List of options to choose from
/// * `default_index` - Optional default selection index
/// * `non_interactive` - If true, returns default or first option
///
/// # Returns
/// The index of the selected option
pub fn prompt_select(
    prompt: &str,
    options: &[&str],
    default_index: Option<usize>,
    non_interactive: bool,
) -> Result<usize> {
    if options.is_empty() {
        return Err(KubegenError::ValidationError(
            "No options provided for selection".to_string(),
        ));
    }

    if non_interactive || !is_interactive() {
        return Ok(default_index.unwrap_or(0));
    }

    let mut select = Select::new().with_prompt(prompt).items(options);

    if let Some(idx) = default_index {
        select = select.default(idx);
    }

    select.interact().map_err(|e| {
        KubegenError::ValidationError(format!("Failed to read selection for '{}': {}", prompt, e))
    })
}

/// Prompt for optional string value, returning None if empty
///
/// # Arguments
/// * `prompt` - The prompt message to display
/// * `non_interactive` - If true, returns None
///
/// # Returns
/// Some(value) if user enters a value, None if empty or non-interactive
pub fn prompt_optional_string(prompt: &str, non_interactive: bool) -> Result<Option<String>> {
    if non_interactive || !is_interactive() {
        return Ok(None);
    }

    let input: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .map_err(|e| {
            KubegenError::ValidationError(format!("Failed to read input for '{}': {}", prompt, e))
        })?;

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompt for string with validation
///
/// # Arguments
/// * `prompt` - The prompt message to display
/// * `validator` - Function that returns Ok(()) if valid, Err(message) if invalid
/// * `default` - Optional default value
/// * `non_interactive` - If true, returns default or error
///
/// # Returns
/// The validated user input
pub fn prompt_validated<F>(
    prompt: &str,
    validator: F,
    default: Option<&str>,
    non_interactive: bool,
) -> Result<String>
where
    F: Fn(&str) -> std::result::Result<(), String> + Clone,
{
    if non_interactive || !is_interactive() {
        if let Some(def) = default {
            validator(def).map_err(KubegenError::ValidationError)?;
            return Ok(def.to_string());
        }
        return Err(KubegenError::ValidationError(format!(
            "Missing required value for '{}' in non-interactive mode",
            prompt
        )));
    }

    let validator_clone = validator.clone();
    let mut input = Input::<String>::new().with_prompt(prompt).validate_with(
        move |input: &String| -> std::result::Result<(), String> { validator_clone(input) },
    );

    if let Some(def) = default {
        input = input.default(def.to_string());
    }

    input.interact_text().map_err(|e| {
        KubegenError::ValidationError(format!("Failed to read input for '{}': {}", prompt, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_string_non_interactive_with_default() {
        let result = prompt_string("Test", Some("default_value"), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default_value");
    }

    #[test]
    fn test_prompt_string_non_interactive_without_default() {
        let result = prompt_string("Test", None, true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-interactive mode"));
    }

    #[test]
    fn test_prompt_confirm_non_interactive() {
        let result = prompt_confirm("Confirm?", true, true);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result = prompt_confirm("Confirm?", false, true);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_prompt_select_non_interactive() {
        let options = &["Option A", "Option B", "Option C"];

        // With default index
        let result = prompt_select("Choose", options, Some(1), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Without default, should return first option
        let result = prompt_select("Choose", options, None, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_prompt_select_empty_options() {
        let options: &[&str] = &[];
        let result = prompt_select("Choose", options, None, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No options"));
    }

    #[test]
    fn test_prompt_optional_string_non_interactive() {
        let result = prompt_optional_string("Optional", true);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_prompt_validated_non_interactive_valid() {
        let validator = |s: &str| {
            if s.len() >= 3 {
                Ok(())
            } else {
                Err("Must be at least 3 characters".to_string())
            }
        };

        let result = prompt_validated("Name", validator, Some("valid"), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid");
    }

    #[test]
    fn test_prompt_validated_non_interactive_invalid_default() {
        let validator = |s: &str| {
            if s.len() >= 3 {
                Ok(())
            } else {
                Err("Must be at least 3 characters".to_string())
            }
        };

        let result = prompt_validated("Name", validator, Some("ab"), true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 3 characters"));
    }

    #[test]
    fn test_prompt_validated_non_interactive_no_default() {
        let validator = |_: &str| Ok(());
        let result = prompt_validated("Name", validator, None, true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-interactive mode"));
    }
}
