use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationKind {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationValue {
    NumericValue(i32),
    StringValue(String),
    BooleanValue(bool),
    MapValue(HashMap<OperationKey, OperationValue>),
    VecValue(Vec<OperationValue>),
}

impl OperationValue {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }
}

impl<K, V> From<Vec<(K, V)>> for OperationValue
where
    K: Into<OperationKey>,
    V: Into<OperationValue>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let map_value = value
            .into_iter()
            .map(|(key, val)| (key.into(), val.into()))
            .collect::<HashMap<OperationKey, OperationValue>>();

        OperationValue::MapValue(map_value)
    }
}

impl From<HashMap<OperationKey, OperationValue>> for OperationValue {
    fn from(value: HashMap<OperationKey, OperationValue>) -> Self {
        OperationValue::MapValue(value)
    }
}

impl<V> From<Vec<V>> for OperationValue
where
    V: Into<OperationValue>,
{
    fn from(value: Vec<V>) -> Self {
        let map_value = value
            .into_iter()
            .map(|val| val.into())
            .collect::<Vec<OperationValue>>();

        OperationValue::VecValue(map_value)
    }
}

impl From<i32> for OperationValue {
    fn from(value: i32) -> Self {
        OperationValue::NumericValue(value)
    }
}

impl From<String> for OperationValue {
    fn from(value: String) -> Self {
        OperationValue::StringValue(value)
    }
}

impl From<&str> for OperationValue {
    fn from(value: &str) -> Self {
        OperationValue::StringValue(value.to_string())
    }
}

impl From<bool> for OperationValue {
    fn from(value: bool) -> Self {
        OperationValue::BooleanValue(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationKey {
    NumericKey(i32),
    StringKey(String),
}

impl Eq for OperationKey {}

impl Hash for OperationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            OperationKey::NumericKey(num) => num.hash(state),
            OperationKey::StringKey(str) => str.hash(state),
        }
    }
}

impl OperationKey {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }
}

impl From<i32> for OperationKey {
    fn from(value: i32) -> Self {
        OperationKey::NumericKey(value)
    }
}

impl From<String> for OperationKey {
    fn from(value: String) -> Self {
        OperationKey::StringKey(value)
    }
}

impl<'a> From<&'a str> for OperationKey {
    fn from(value: &'a str) -> Self {
        OperationKey::StringKey(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub time: Instant,
    pub kind: OperationKind,
    pub key: OperationKey,
    pub current_value: Option<OperationValue>,
    pub prev_value: Option<OperationValue>,
}

#[derive(Debug, Clone)]
pub struct LogOperator {
    pub operations: Vec<Operation>,
}

impl LogOperator {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn insert<K, V>(&mut self, key: K, current_value: V)
    where
        K: Into<OperationKey>,
        V: Into<OperationValue>,
    {
        let key = key.into();
        let current_value = Some(current_value.into());

        self.operations.push(Operation {
            time: Instant::now(),
            kind: OperationKind::Insert,
            key,
            current_value,
            prev_value: None,
        });
    }

    pub fn update<K, V>(&mut self, key: K, current_value: V, prev_value: Option<V>)
    where
        K: Into<OperationKey>,
        V: Into<OperationValue>,
    {
        let key = key.into();
        let current_value = Some(current_value.into());
        let prev_value = prev_value.map(Into::into);

        self.operations.push(Operation {
            time: Instant::now(),
            kind: OperationKind::Update,
            key,
            current_value,
            prev_value,
        });
    }

    pub fn delete<K>(&mut self, key: K)
    where
        K: Into<OperationKey>,
    {
        let key = key.into();

        self.operations.push(Operation {
            time: Instant::now(),
            kind: OperationKind::Delete,
            key,
            current_value: None,
            prev_value: None
        });
    }
}
