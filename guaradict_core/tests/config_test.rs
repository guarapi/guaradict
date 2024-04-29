use guaradict_core::config::parse_config_file;

#[test]
fn test_parse_config_file_multi_primary_multi_replica() {
    let config = parse_config_file("tests/fixtures/multi-primary-multi-replica.yaml").unwrap();
    println!("{:#?}", config);
    assert_eq!(config.node_type, "primary");
    assert_eq!(config.name, "primary-node-1");
    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 13141);
    assert!(config.database.is_none());
    assert!(config.replicas.is_some());

    let replicas = config.replicas.unwrap();
    assert_eq!(replicas.len(), 3);

    let replica1 = &replicas[0];
    assert_eq!(replica1.node_type, "primary");
    assert_eq!(replica1.name, "primary-node-2");
    assert_eq!(replica1.ip, "127.0.0.1");
    assert_eq!(replica1.host, "127.0.0.1");
    assert_eq!(replica1.port, 13142);
    assert!(replica1.database.is_none());

    let replica2 = &replicas[1];
    assert_eq!(replica2.node_type, "replica");
    assert_eq!(replica2.name, "replica-node-1");
    assert_eq!(replica2.ip, "127.0.0.1");
    assert_eq!(replica2.host, "127.0.0.1");
    assert_eq!(replica2.port, 13143);
    assert!(replica2.database.is_none());

    let replica3 = &replicas[2];
    assert_eq!(replica3.node_type, "replica");
    assert_eq!(replica3.name, "replica-node-1");
    assert_eq!(replica3.ip, "127.0.0.1");
    assert_eq!(replica3.host, "127.0.0.1");
    assert_eq!(replica3.port, 13144);
    assert!(replica2.database.is_none());
}

#[test]
fn test_parse_config_file_primary_multi_replica() {
    let config = parse_config_file("tests/fixtures/primary-multi-replica.yaml").unwrap();
    assert_eq!(config.node_type, "primary");
    assert_eq!(config.name, "primary-node-1");
    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 13141);
    assert!(config.database.is_none());
    assert!(config.replicas.is_some());

    let replicas = config.replicas.unwrap();
    assert_eq!(replicas.len(), 2);

    let replica1 = &replicas[0];
    assert_eq!(replica1.node_type, "replica");
    assert_eq!(replica1.name, "replica-node-1");
    assert_eq!(replica1.ip, "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff");
    assert_eq!(replica1.host, "127.0.0.1");
    assert_eq!(replica1.port, 13142);
    assert!(replica1.database.is_none());

    let replica2 = &replicas[1];
    assert_eq!(replica2.node_type, "replica");
    assert_eq!(replica2.name, "replica-node-2");
    assert_eq!(replica2.ip, "127.0.0.1");
    assert_eq!(replica2.host, "127.0.0.1");
    assert_eq!(replica2.port, 13143);
    assert!(replica2.database.is_none());
}

#[test]
fn test_parse_config_file_primary_replica() {
    let config = parse_config_file("tests/fixtures/primary-replica.yaml").unwrap();
    assert_eq!(config.node_type, "primary");
    assert_eq!(config.name, "primary-node");
    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 13141);
    assert_eq!(config.database, Some(String::from("my-database")));
    assert!(config.replicas.is_some());

    let replicas = config.replicas.unwrap();
    assert_eq!(replicas.len(), 1);

    let replica = &replicas[0];
    assert_eq!(replica.node_type, "replica");
    assert_eq!(replica.name, "replica-node-1");
    assert_eq!(replica.ip, "127.0.0.1");
    assert_eq!(replica.host, "127.0.0.1");
    assert_eq!(replica.port, 13142);
    assert_eq!(replica.database, Some(String::from("my-database")));
}

#[test]
fn test_parse_config_file_replica() {
    let config = parse_config_file("tests/fixtures/replica.yaml").unwrap();
    assert_eq!(config.node_type, "replica");
    assert_eq!(config.name, "replica-node-1");
    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 13142);
    assert_eq!(config.database, Some(String::from("my-database")));
    assert!(config.replicas.is_none());
}
