//! configs git helper

use super::utils::repo;
use crate::error::Result;
use git2::Repository;
use scopetime::scope_time;

/// Possible options for the status.showUntrackedFiles
///
/// see https://git-scm.com/docs/git-config#Documentation/git-config.txt-statusshowUntrackedFiles
pub enum ShowUntrackedFilesConfig {
    /// Don't include any untracked fiels
    No,
    /// Show untracked files and directories, but do not recurse into untracked directories to find files
    Normal,
    /// Show all untracked files recursively
    All,
}

impl ShowUntrackedFilesConfig {
    /// should include untracked files?
    pub const fn include_untracked(&self) -> bool {
        matches!(self, Self::Normal | Self::All)
    }

    /// should recurse untracked dirs?
    pub const fn recurse_untracked_dirs(&self) -> bool {
        matches!(self, Self::All)
    }
}

/// Get the showUntrackedFiles config option for a Repository
pub fn untracked_files_config(
    repo: &Repository,
) -> Result<ShowUntrackedFilesConfig> {
    let show_untracked_files =
        get_config_string_repo(repo, "status.showUntrackedFiles")?;

    if let Some(show_untracked_files) = show_untracked_files {
        if &show_untracked_files == "no" {
            return Ok(ShowUntrackedFilesConfig::No);
        } else if &show_untracked_files == "normal" {
            return Ok(ShowUntrackedFilesConfig::Normal);
        }
    }

    Ok(ShowUntrackedFilesConfig::All)
}

/// get string from config
pub fn get_config_string(
    repo_path: &str,
    key: &str,
) -> Result<Option<String>> {
    let repo = repo(repo_path)?;
    get_config_string_repo(&repo, key)
}

/// get string from config (from Repository object)
pub fn get_config_string_repo(
    repo: &Repository,
    key: &str,
) -> Result<Option<String>> {
    scope_time!("get_config_string_repo");

    let cfg = repo.config()?;

    // this code doesnt match what the doc says regarding what
    // gets returned when but it actually works
    let entry_res = cfg.get_entry(key);

    let entry = match entry_res {
        Ok(ent) => ent,
        Err(_) => return Ok(None),
    };

    if entry.has_value() {
        Ok(entry.value().map(std::string::ToString::to_string))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::tests::repo_init;

    #[test]
    fn test_get_config() {
        let bad_dir_cfg =
            get_config_string("oodly_noodly", "this.doesnt.exist");
        assert!(bad_dir_cfg.is_err());

        let (_td, repo) = repo_init().unwrap();
        let path = repo.path();
        let rpath = path.as_os_str().to_str().unwrap();
        let bad_cfg = get_config_string(rpath, "this.doesnt.exist");
        assert!(bad_cfg.is_ok());
        assert!(bad_cfg.unwrap().is_none());
        // repo init sets user.name
        let good_cfg = get_config_string(rpath, "user.name");
        assert!(good_cfg.is_ok());
        assert!(good_cfg.unwrap().is_some());
    }
}