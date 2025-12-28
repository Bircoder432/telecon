use std::sync::Arc;
use telecon::{
    config::Config,
    parser::{
        Service,
        handler::{HandlerConfig, HandlerRegistry, load_handlers},
        parse_tree,
        tree::Node,
    },
    runner::{run_custom_handler, run_service},
    utils::find_node,
};
use tokio::sync::RwLock;

#[tokio::test]
async fn test_config_load() {
    // создаем тестовый config.toml
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("config.toml");
    std::fs::write(
        &path,
        r#"
        token = "test-token"
        owner_id = 123
    "#,
    )
    .unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    assert_eq!(config.token, "test-token");
    assert_eq!(config.owner_id, 123);
}

#[tokio::test]
async fn test_parse_tree_and_find_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let service_path = temp_dir.path().join("test_service.toml");
    std::fs::write(
        &service_path,
        r#"
        title = "Test Service"
        name = "test_service"
        command = "echo"
        args = ["hello"]
    "#,
    )
    .unwrap();

    let tree = parse_tree(temp_dir.path().to_str().unwrap(), "root");
    match &tree {
        Node::Folder(folder) => {
            assert_eq!(folder.title, "root");
            assert!(folder.children.contains_key("test_service"));
        }
        _ => panic!("Root should be a folder"),
    }

    let node = find_node(&tree, "test_service").unwrap();
    match node {
        Node::Service(s) => assert_eq!(s.name, "test_service"),
        _ => panic!("Node should be service"),
    }
}

#[tokio::test]
async fn test_handler_registry_load() {
    let temp_dir = tempfile::tempdir().unwrap();
    let handler_path = temp_dir.path().join("handler.toml");
    std::fs::write(
        &handler_path,
        r#"
        name = "my_handler"
        exec = "echo"
        args = ["hello"]
    "#,
    )
    .unwrap();

    let registry: HandlerRegistry = load_handlers(temp_dir.path().to_str().unwrap());
    assert!(registry.handlers.contains_key("my_handler"));
    let handler: &HandlerConfig = registry.handlers.get("my_handler").unwrap();
    assert_eq!(handler.exec, "echo");
}

#[tokio::test]
async fn test_run_service_and_handler() {
    let service = Service {
        title: "Echo Service".into(),
        name: "echo".into(),
        command: "echo".into(),
        args: vec!["hello".into()],
    };

    run_service(&service).await;

    let handler = telecon::parser::handler::HandlerConfig {
        name: "handler".into(),
        exec: "echo".into(),
        args: vec!["world".into()],
    };

    run_custom_handler(&handler).await;
}
