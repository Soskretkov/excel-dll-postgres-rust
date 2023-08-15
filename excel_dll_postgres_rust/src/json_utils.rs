use indexmap::IndexMap;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use tokio_postgres::{types::Type, Column};
use chrono::NaiveDate;
use serde_json::{json, Value};
use tokio_postgres::Row;

// сиротское правило не дает реализовать трейт Serialize непосредственно на IndexMap
pub struct OrderedJson(IndexMap<String, Value>);

impl Serialize for OrderedJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl OrderedJson {
    pub fn new() -> Self {
        OrderedJson(IndexMap::new())
    }
    pub fn insert(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
    }
}

pub fn convert_to_serde_json_type(row: &Row, column: &Column) -> Value {
    //потенциально добавить: pg_lsn
    match *column.type_() {
        Type::BOOL => match row.try_get::<_, bool>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::CHAR => match row.try_get::<_, i8>(column.name()) {
            Ok(v) => {
                let ch = char::from_u32(v as u32).unwrap_or('\0');
                json!(ch.to_string())
            }
            Err(_) => Value::Null,
        },
        Type::INT2 => match row.try_get::<_, i16>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::INT4 => match row.try_get::<_, i32>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::OID => match row.try_get::<_, u32>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::INT8 => match row.try_get::<_, i64>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::FLOAT4 => match row.try_get::<_, f32>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::FLOAT8 => match row.try_get::<_, f64>(column.name()) {
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
        Type::DATE => match row.try_get::<_, NaiveDate>(column.name()) {
            Ok(v) => {
                // let base_date = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                // let duration = v.signed_duration_since(base_date);
                // json!(duration.num_days())
                json!(v.format("%Y-%m-%d").to_string())
            }
            Err(_) => Value::Null,
        },
        _ => match row.try_get::<_, String>(column.name()) {
            //VARCHAR, CHAR(n), TEXT, CITEXT, NAME
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
    }
}