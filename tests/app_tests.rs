use octo_work::app::{App, Modal, Pane};
use octo_work::config::Config;
use octo_work::models::Scope;
use octo_work::workspace;
use crossterm::event::KeyCode;
use tempfile::tempdir;

#[test]
fn test_app_init_and_workspace_lifecycle() {
    let tmp = tempdir().unwrap();
    let config = Config::new(tmp.path());

    // Initially directories do not exist, App should show InitDirs modal
    let mut app = App::new(config.clone());
    assert!(matches!(app.modal, Some(Modal::InitDirs { .. })));

    // Press Enter to initialize
    app.handle_key(KeyCode::Enter);
    assert!(app.modal.is_none());
    assert!(workspace::require_dirs(&config).is_ok());

    // Add workspace via modal
    app.handle_key(KeyCode::Char('a'));
    assert!(matches!(app.modal, Some(Modal::AddWorkspace { .. })));

    // Type "feature-xyz"
    for c in "feature-xyz".chars() {
        app.handle_key(KeyCode::Char(c));
    }
    app.handle_key(KeyCode::Enter);

    // Verify workspace was created and selected
    assert_eq!(app.filtered_workspaces.len(), 1);
    assert_eq!(app.selected_workspace().unwrap().name, "feature-xyz");
    assert_eq!(app.scope, Scope::Active);

    // Create a file in workspace
    app.focused_pane = Pane::Files;
    app.handle_key(KeyCode::Char('n'));
    assert!(matches!(app.modal, Some(Modal::NewFile { .. })));

    for c in "prompt.md".chars() {
        app.handle_key(KeyCode::Char(c));
    }
    app.handle_key(KeyCode::Enter);

    assert_eq!(app.files.len(), 1);
    assert_eq!(app.selected_file().unwrap().rel_path, "prompt.md");

    // Archive the workspace
    app.focused_pane = Pane::Workspaces;
    app.handle_key(KeyCode::Char('A'));
    assert!(matches!(app.modal, Some(Modal::ConfirmArchive { .. })));
    app.handle_key(KeyCode::Char('y'));

    // Active workspaces should now be empty
    assert_eq!(app.filtered_workspaces.len(), 0);

    // Toggle scope to Archive
    app.handle_key(KeyCode::Char('s'));
    assert_eq!(app.scope, Scope::Archive);
    assert_eq!(app.filtered_workspaces.len(), 1);
    assert_eq!(app.selected_workspace().unwrap().name, "feature-xyz");

    // Restore workspace
    app.handle_key(KeyCode::Char('A'));
    assert!(matches!(app.modal, Some(Modal::ConfirmRestore { .. })));
    app.handle_key(KeyCode::Enter);

    // Scope automatically switched to Active with restored workspace
    assert_eq!(app.scope, Scope::Active);
    assert_eq!(app.filtered_workspaces.len(), 1);
    assert_eq!(app.selected_workspace().unwrap().name, "feature-xyz");
}

#[test]
fn test_app_search_filter() {
    let tmp = tempdir().unwrap();
    let config = Config::new(tmp.path());
    workspace::init_dirs(&config).unwrap();
    workspace::add_workspace(&config, "alpha-task").unwrap();
    workspace::add_workspace(&config, "beta-task").unwrap();
    workspace::add_workspace(&config, "gamma-item").unwrap();

    let mut app = App::new(config);
    assert_eq!(app.filtered_workspaces.len(), 3);

    // Open search modal
    app.handle_key(KeyCode::Char('/'));
    for c in "task".chars() {
        app.handle_key(KeyCode::Char(c));
    }
    app.handle_key(KeyCode::Enter);

    assert_eq!(app.filtered_workspaces.len(), 2);
    assert!(app.filtered_workspaces.iter().all(|w| w.name.contains("task")));

    // Clear filter
    app.handle_key(KeyCode::Esc);
    assert_eq!(app.filtered_workspaces.len(), 3);
}

#[test]
fn test_app_init_dirs_with_gitignore_toggle() {
    use std::fs;

    let tmp = tempdir().unwrap();
    let root = tmp.path();
    // Simulate git repo
    fs::create_dir(root.join(".git")).unwrap();

    let config = Config::new(root);
    let mut app = App::new(config.clone());

    if let Some(Modal::InitDirs {
        add_to_gitignore,
        git_detected,
    }) = app.modal
    {
        assert!(git_detected);
        assert!(add_to_gitignore);
    } else {
        panic!("Expected Modal::InitDirs");
    }

    // Toggle spacebar to uncheck
    app.handle_key(KeyCode::Char(' '));
    if let Some(Modal::InitDirs {
        add_to_gitignore, ..
    }) = app.modal
    {
        assert!(!add_to_gitignore);
    } else {
        panic!("Expected Modal::InitDirs after toggle");
    }

    // Toggle spacebar back to checked
    app.handle_key(KeyCode::Char(' '));
    if let Some(Modal::InitDirs {
        add_to_gitignore, ..
    }) = app.modal
    {
        assert!(add_to_gitignore);
    } else {
        panic!("Expected Modal::InitDirs after toggle");
    }

    // Press Enter to initialize
    app.handle_key(KeyCode::Enter);
    assert!(app.modal.is_none());
    assert!(workspace::require_dirs(&config).is_ok());

    // Verify .gitignore was created with patterns
    let gitignore_content = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert_eq!(gitignore_content, "workspace/\narchive/\n");
    assert!(app.status.as_ref().unwrap().0.contains("Added to .gitignore"));
}
