use std::process::Command;

use anyhow::{bail, Context, Result};

const REPOSITORY_URL: &str = "https://github.com/maximbezd99/asapi-cli";

pub fn run() -> Result<()> {
    let source = format!(
        "{REPOSITORY_URL}/tree/{}/skills/app-store-research",
        env!("CARGO_PKG_VERSION")
    );
    let status = Command::new("npx")
        .args(["--yes", "skills", "add"])
        .arg(&source)
        .status()
        .context("could not run npx; install Node.js and npm, then try again")?;
    if !status.success() {
        match status.code() {
            Some(code) => bail!("skill installer exited with status {code}"),
            None => bail!("skill installer was terminated before it finished"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn skill_declares_binary_version() {
        let expected = format!(
            "This skill is designed for `asapi` CLI version `{}`.",
            env!("CARGO_PKG_VERSION")
        );
        assert!(include_str!("../../../skills/app-store-research/SKILL.md").contains(&expected));
    }
}
