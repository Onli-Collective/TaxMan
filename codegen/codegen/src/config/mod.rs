// I think I handled each possible error but I might've missed something
// Idk if you want the errors handled in some other way

/*

Note for ease of use: I typically write code s.t.

1. any structs a func is dependant upon are above that func,
2. any structs a struct is dependant upon are below that struct,
3. any funcs a func is dependant upon are below that func

*/

/*
Use statements
*/

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cargo_toml::{Inheritable, Manifest};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

/*
Public types
*/

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceConfig {
    pub common: Config<metadata::Common>,
    pub apis: BTreeMap<String, Config<metadata::Api>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config<T> {
    path: PathBuf,
    name: String,
    version: cargo_toml::SemVer,
    codegen: T,
}

pub mod metadata {
    use serde::Deserialize;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Common;

    impl<'de> Deserialize<'de> for Common {
        fn deserialize<D>(_: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(Self)
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct Api {
        pub url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Root {
        #[serde(rename = "$common")]
        pub common: String,
        #[serde(default)]
        pub apis: Vec<String>,
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    // External errors
    #[error("could not read `{}`: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not parse `{}`: {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: cargo_toml::Error,
    },

    // Internal errors
    #[error("`{}` is missing `[workspace]` table", path.display())]
    MissingWorkspace { path: PathBuf },

    #[error("`{}` is missing `[workspace.metadata.codegen]`", path.display())]
    MissingCodegenMetadata { path: PathBuf },

    #[error("`{}`: `[workspace.metadata.codegen]` is missing common crate key", path.display())]
    MissingCommon { path: PathBuf },

    #[error("`{}`: `[workspace.metadata.codegen]` must contain at least one api crate key", path.display())]
    MissingApis { path: PathBuf },

    #[error("`{}` is missing `[package]` table", path.display())]
    MissingPackage { path: PathBuf },

    #[error("`{}`: `package.version` is inherited from the workspace (`version.workspace = true`), which isn't supported; set it explicitly", path.display())]
    InheritedVersion { path: PathBuf },

    #[error("`{}` is missing `[package.metadata.codegen]`", path.display())]
    MissingApiMetadata { path: PathBuf },

    #[error("`{}`: `[package.metadata.codegen]` is missing `url`", path.display())]
    MissingUrl { path: PathBuf },

    #[error("API crates at `{}` and `{}` both have the name `{name}`", api1.display(), api2.display())]
    DuplicateApi {
        name: String,
        api1: PathBuf,
        api2: PathBuf,
    },
}

#[derive(Debug, Deserialize)]
struct WithMetadata<T> {
    codegen: T,
}

impl WorkspaceConfig {
    pub fn read_workspace<A: AsRef<Path>>(root: A) -> Result<WorkspaceConfig, ConfigError> {
        let root = root.as_ref();

        let manifest_path = root.join("Cargo.toml");
        let manifest: Manifest<WithMetadata<metadata::Root>> = load_manifest(&manifest_path)?;

        dbg!(&manifest);

        let metadata = manifest
            .workspace
            .ok_or_else(|| ConfigError::MissingWorkspace {
                path: manifest_path.clone(),
            })?
            .metadata
            .ok_or_else(|| ConfigError::MissingCodegenMetadata {
                path: manifest_path.clone(),
            })?;

        let codegen = metadata.codegen;
        // TODO: check if serde play nicely with errors
        // .ok_or_else(|| ConfigError::MissingCodegenMetadata {
        //     path: manifest_path.clone(),
        // })?;

        let common_relative = codegen.common;
        // .ok_or_else(|| ConfigError::MissingCommon {
        //     path: manifest_path.clone(),
        // })?;
        let common = Config::read_crate(&root.join(common_relative))?;

        if codegen.apis.is_empty() {
            return Err(ConfigError::MissingApis {
                path: manifest_path,
            });
        }

        let mut apis: BTreeMap<String, Config<metadata::Api>> = BTreeMap::new();
        for api_relative in codegen.apis {
            let api = Config::read_crate(&root.join(api_relative))?;

            // Catch duplicates
            if let Some(existing) = apis.get(&api.name) {
                return Err(ConfigError::DuplicateApi {
                    name: api.name,
                    api1: existing.path.clone(),
                    api2: api.path,
                });
            }
            apis.insert(api.name.clone(), api);
        }

        Ok(WorkspaceConfig { common, apis })
    }
}

impl<T: DeserializeOwned> Config<T> {
    pub fn read_crate<A: AsRef<Path>>(dir: A) -> Result<Config<T>, ConfigError> {
        let dir = dir.as_ref();
        let manifest_path = dir.join("Cargo.toml");

        let manifest: Manifest<WithMetadata<T>> = load_manifest(&manifest_path)?;

        // TODO: check if serde error is more useful.
        let package = manifest
            .package
            .ok_or_else(|| ConfigError::MissingPackage {
                path: manifest_path.clone(),
            })?;

        // API and Common crates must have explicit versions declared in their Cargo.toml manifests to ensure
        // they match with the version number declared in the OpenAPI document
        // (and no point for the common crate since it's on its own).
        let version = match package.version {
            Inheritable::Set(v) => v,
            Inheritable::Inherited => {
                return Err(ConfigError::InheritedVersion {
                    path: manifest_path,
                });
            }
        };

        // Neat solution checks if metadata in !None and then takes codegen
        let codegen =
            package
                .metadata
                .map(|m| m.codegen)
                .ok_or_else(|| ConfigError::MissingApiMetadata {
                    path: manifest_path.clone(),
                })?;

        Ok(Config {
            name: package.name,
            version,
            path: dir.to_owned(),
            codegen,
        })
    }
}

fn load_manifest<M: DeserializeOwned>(path: &Path) -> Result<Manifest<M>, ConfigError> {
    let file_data = std::fs::read(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Manifest::<M>::from_slice_with_metadata(&file_data).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use crate::config::WorkspaceConfig;

    #[test]
    fn a() {
        let current_dir = std::env::current_dir().unwrap();
        println!("{}", current_dir.display());
        let config = WorkspaceConfig::read_workspace(current_dir);
        println!("{config:?}")
    }
}
