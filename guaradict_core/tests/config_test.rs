use guaradict_core::config::parse_config_file;

#[test]
fn test_parse_config_file_multi_primary_multi_replica() {
    let config = parse_config_file("tests/fixtures/multi-primary-multi-replica.yaml").unwrap();
    assert_eq!(config.primary.is_some(), true);
    assert_eq!(config.journal.is_some(), true);

    let primary = config.primary.unwrap();
    assert_eq!(primary.len(), 2);

    let node1 = &primary[0];
    assert_eq!(node1.name, "primary-node-1");
    assert_eq!(node1.ip, "::0");
    assert_eq!(node1.host, "::0");
    assert_eq!(node1.database, "my-database-1");
    assert!(node1.replicas.is_some());

    let replica1 = &node1.replicas.as_ref().unwrap()[0];
    assert_eq!(replica1.name, "replica-node-1");
    assert_eq!(replica1.ip, "127.0.0.1");
    assert_eq!(replica1.host, "127.0.0.1");
    assert_eq!(replica1.database, "my-database-1");

    let node2 = &primary[1];
    assert_eq!(node2.name, "primary-node-2");
    assert_eq!(node2.ip, "::0");
    assert_eq!(node2.host, "::0");
    assert_eq!(node2.database, "my-database-2");
    assert!(node2.replicas.is_some());

    let replica2 = &node2.replicas.as_ref().unwrap()[0];
    assert_eq!(replica2.name, "replica-node-2");
    assert_eq!(replica2.ip, "127.0.0.1");
    assert_eq!(replica2.host, "127.0.0.1");
    assert_eq!(replica2.database, "my-database-2");

    let journal = config.journal.unwrap();
    assert_eq!(journal.strategy, "sync");
}


#[test]
fn test_parse_config_file_primary_multi_replica() {
    let config = parse_config_file("tests/fixtures/primary-multi-replica.yaml").unwrap();
    assert_eq!(config.primary.is_some(), true);
    assert_eq!(config.journal.is_some(), true);

    let primary = config.primary.unwrap();
    assert_eq!(primary.len(), 1);

    let node = &primary[0];
    assert_eq!(node.name, "primary-node");
    assert_eq!(node.ip, "::0");
    assert_eq!(node.host, "::0");
    assert_eq!(node.database, "my-database");
    assert!(node.replicas.is_some());

    let replicas = node.replicas.as_ref().unwrap();
    assert_eq!(replicas.len(), 2);

    let replica1 = &replicas[0];
    assert_eq!(replica1.name, "replica-node-1");
    assert_eq!(replica1.ip, "127.0.0.1");
    assert_eq!(replica1.host, "127.0.0.1");
    assert_eq!(replica1.database, "my-database");

    let replica2 = &replicas[1];
    assert_eq!(replica2.name, "replica-node-2");
    assert_eq!(replica2.ip, "::1");
    assert_eq!(replica2.host, "::1");
    assert_eq!(replica2.database, "my-database");

    let journal = config.journal.unwrap();
    assert_eq!(journal.strategy, "snapshot_log");
}


#[test]
fn test_parse_config_file_primary_replica() {
    let config = parse_config_file("tests/fixtures/primary-replica.yaml").unwrap();
    assert_eq!(config.primary.is_some(), true);
    assert_eq!(config.journal.is_some(), true);

    let primary = config.primary.unwrap();
    assert_eq!(primary.len(), 1);

    let node = &primary[0];
    assert_eq!(node.name, "primary-node");
    assert_eq!(node.ip, "::0");
    assert_eq!(node.host, "::0");
    assert_eq!(node.database, "my-database");
    assert!(node.replicas.is_some());

    let replicas = node.replicas.as_ref().unwrap();
    assert_eq!(replicas.len(), 1);

    let replica = &replicas[0];
    assert_eq!(replica.name, "replica-node-1");
    assert_eq!(replica.ip, "127.0.0.1");
    assert_eq!(replica.host, "127.0.0.1");
    assert_eq!(replica.database, "my-database");

    let journal = config.journal.unwrap();
    assert_eq!(journal.strategy, "async");
}
