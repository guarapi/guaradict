use guaradict_core::replica::{LogOperator, OperationKind};

#[test]
fn test_insert_operation() {
    let mut log_operator = LogOperator::new();
    log_operator.insert("key1", 42);

    assert_eq!(log_operator.operations.len(), 1);

    let operation = &log_operator.operations[0];
    assert_eq!(operation.kind, OperationKind::Insert);
    assert_eq!(operation.key, "key1".into());
    assert_eq!(operation.current_value, Some(42.into()));
    assert_eq!(operation.prev_value, None);
}

#[test]
fn test_update_operation() {
    let mut log_operator = LogOperator::new();
    log_operator.update("key2", "new_value", Some("old_value"));

    assert_eq!(log_operator.operations.len(), 1);

    let operation = &log_operator.operations[0];
    assert_eq!(operation.kind, OperationKind::Update);
    assert_eq!(operation.key, "key2".into());
    assert_eq!(operation.current_value, Some("new_value".into()));
    assert_eq!(operation.prev_value, Some("old_value".into()));
}

#[test]
fn test_delete_operation() {
    let mut log_operator = LogOperator::new();
    log_operator.delete("key3");

    assert_eq!(log_operator.operations.len(), 1);

    let operation = &log_operator.operations[0];
    assert_eq!(operation.kind, OperationKind::Delete);
    assert_eq!(operation.key, "key3".into());
    assert_eq!(operation.current_value, None);
    assert_eq!(operation.prev_value, None);
}
