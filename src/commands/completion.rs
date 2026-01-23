//! Implementation of the `kubegen completion` command
//!
//! Generates shell completion scripts for various shells.

use std::io;

use clap::CommandFactory;
use clap_complete::{self, Generator};

use crate::cli::{Cli, CompletionArgs, Shell};

/// Execute the `kubegen completion` command
pub fn execute_completion(args: &CompletionArgs) {
    let mut cmd = Cli::command();
    let shell = match args.shell {
        Shell::Bash => clap_complete::Shell::Bash,
        Shell::Zsh => clap_complete::Shell::Zsh,
        Shell::Fish => clap_complete::Shell::Fish,
        Shell::PowerShell => clap_complete::Shell::PowerShell,
        Shell::Elvish => clap_complete::Shell::Elvish,
    };

    print_completions(shell, &mut cmd);
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    clap_complete::generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_bash() {
        let mut cmd = Cli::command();
        let shell = clap_complete::Shell::Bash;
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, "kubegen", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("kubegen"));
        assert!(output.contains("complete"));
    }

    #[test]
    fn test_completion_zsh() {
        let mut cmd = Cli::command();
        let shell = clap_complete::Shell::Zsh;
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, "kubegen", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("kubegen"));
        assert!(output.contains("compdef") || output.contains("_arguments"));
    }

    #[test]
    fn test_completion_fish() {
        let mut cmd = Cli::command();
        let shell = clap_complete::Shell::Fish;
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, "kubegen", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("kubegen"));
        assert!(output.contains("complete"));
    }

    #[test]
    fn test_completion_powershell() {
        let mut cmd = Cli::command();
        let shell = clap_complete::Shell::PowerShell;
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, "kubegen", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("kubegen"));
        assert!(output.contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn test_completion_elvish() {
        let mut cmd = Cli::command();
        let shell = clap_complete::Shell::Elvish;
        let mut buf = Vec::new();
        clap_complete::generate(shell, &mut cmd, "kubegen", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("kubegen"));
    }
}
