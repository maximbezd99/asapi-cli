use std::process::Command;

use anyhow::{bail, Context, Result};

const REPOSITORY_URL: &str = "https://github.com/maximbezd99/asapi-cli";

pub fn run() -> Result<()> {
    let source = source_for_version(env!("CARGO_PKG_VERSION"));
    let status = Command::new("npx")
        .args(["--yes", "skills", "add"])
        .arg(&source)
        .status()
        .context("could not run npx; install Node.js and npm, then try again")?;

    if !status.success() {
        match status.code() {
            Some(code) => bail!(
                "skill installer exited with status {code}; verify that release tag {} exists at {REPOSITORY_URL}",
                env!("CARGO_PKG_VERSION")
            ),
            None => bail!("skill installer was terminated before it finished"),
        }
    }

    Ok(())
}

fn source_for_version(version: &str) -> String {
    format!("{REPOSITORY_URL}/tree/{version}/skills/asapi")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_source_is_pinned_to_binary_version() {
        assert_eq!(
            source_for_version("1.2.3"),
            "https://github.com/maximbezd99/asapi-cli/tree/1.2.3/skills/asapi"
        );
    }

    #[test]
    fn skill_declares_binary_version() {
        let expected = format!(
            "This skill is designed for `asapi` CLI version `{}`.",
            env!("CARGO_PKG_VERSION")
        );
        assert!(
            include_str!("../../skills/asapi/SKILL.md").contains(&expected),
            "SKILL.md must declare compatibility with {}",
            env!("CARGO_PKG_VERSION")
        );
    }
}
