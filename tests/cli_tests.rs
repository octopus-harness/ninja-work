use ninja_work::cli::{self, Cli, Commands};
use ninja_work::config::Config;
use ninja_work::workspace;
use tempfile::tempdir;

#[test]
fn test_cli_lifecycle() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let config = Config::new(&root);

    // Init
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Init),
        },
        config.clone(),
    )
    .unwrap();
    assert!(workspace::require_dirs(&config).is_ok());

    // Add
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Add {
                name: Some("test-project".into()),
            }),
        },
        config.clone(),
    )
    .unwrap();
    assert!(config.workspace_dir.join("test-project").is_dir());

    // Path
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Path {
                name: Some("test-project".into()),
                archived: false,
            }),
        },
        config.clone(),
    )
    .unwrap();

    // Rename
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Rename {
                old_name: "test-project".into(),
                new_name: "renamed-project".into(),
                archived: false,
            }),
        },
        config.clone(),
    )
    .unwrap();
    assert!(!config.workspace_dir.join("test-project").exists());
    assert!(config.workspace_dir.join("renamed-project").is_dir());

    // Archive
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Archive {
                name: Some("renamed-project".into()),
            }),
        },
        config.clone(),
    )
    .unwrap();
    assert!(!config.workspace_dir.join("renamed-project").exists());
    assert!(config.archive_dir.join("renamed-project").is_dir());

    // Restore
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Restore {
                name: Some("renamed-project".into()),
                target: None,
            }),
        },
        config.clone(),
    )
    .unwrap();
    assert!(config.workspace_dir.join("renamed-project").is_dir());
    assert!(!config.archive_dir.join("renamed-project").exists());

    // Delete with -y
    cli::run_cli(
        Cli {
            root: Some(root.clone()),
            command: Some(Commands::Delete {
                name: Some("renamed-project".into()),
                archived: false,
                yes: true,
            }),
        },
        config.clone(),
    )
    .unwrap();
    assert!(!config.workspace_dir.join("renamed-project").exists());
}
