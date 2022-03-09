use anyhow::Result;
use vergen::{vergen, Config, SemverKind};

fn main() -> Result<()> {
    let mut config = Config::default();
    // *config.git_mut().enabled_mut() = false;
    *config.git_mut().semver_kind_mut() = SemverKind::Lightweight;

    vergen(config)
}
