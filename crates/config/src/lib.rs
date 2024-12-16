mod battery;
mod types;
mod urgency;

use std::path::{Path, PathBuf};

use merge2::Merge;

pub use battery::*;
pub use clap::Parser;
pub use directories::ProjectDirs;
pub use types::*;
pub use urgency::*;

#[inline]
pub(crate) fn swap_option<T>(left: &mut Option<T>, right: &mut Option<T>) {
    if left.is_none() || right.is_some() {
        core::mem::swap(left, right);
    }
}

pub fn write_default(path: impl AsRef<Path>) {
    // This need fail if cannot write
    std::fs::write(path, toml::to_string_pretty(&Config::default()).unwrap()).unwrap();
}

pub fn get_config(args: &mut Config, project: &ProjectDirs) -> Option<(PathBuf, Config)> {
    let config_path = if let Some(path) = args.config.as_ref() {
        // tracing::trace!("Loading custom path");
        println!("Loading custom path");
        path.clone()
    } else {
        let config_path = project.config_dir();

        _ = std::fs::create_dir_all(config_path);

        // tracing::trace!("Loading global config");
        println!("Loading global config");
        config_path.join("config.toml")
    };
    // tracing::info!("Reading configs from path: {config_path:?}");
    println!("Reading configs from path: {config_path:?}");

    if let Ok(cfg_content) = std::fs::read_to_string(&config_path) {
        // tracing::debug!("Merging from config file");
        println!("Merging from config file");
        let mut config: Config = toml::from_str(&cfg_content).ok()?;
        config.merge(args);
        return Some((config_path, config));
    }
    let mut config = Config::default();
    config.merge(args);

    Some((config_path, config))
}
