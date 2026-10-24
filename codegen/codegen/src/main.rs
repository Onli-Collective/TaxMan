use taxman::config::{ConfigError, WorkspaceConfig};

fn main() -> Result<(), ConfigError> {
    let current_dir = std::env::current_dir().expect("invalid or inaccessible current directory");
    println!("{}", current_dir.display());
    let config = WorkspaceConfig::read_workspace(current_dir)?;
    println!("{config:?}");
    Ok(())
}
