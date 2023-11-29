// Назначение модуля кратко:  предоставляет утилиты и функций, которые упрощают работу с JSON.
// Подробное описание: модуль может содержать функции безопасного парсинга JSON, преобразования данных
// в JSON и обратно, валидации JSON-структур и так далее. Этот модуль может быть полезен в разных
// частях приложения, а не только в контексте API. По этой причине код отделен от модуля api.rs с
// целью соблюдения принципа единственной ответственности.
use super::Error;
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use tokio_postgres::Row;
use tokio_postgres::{types::Type, Column};

// тип-обертка, сиротское правило не дает реализовать трейт Serialize для IndexMap
// IndexMap выбран потому что сохраняет порядок в которой вносятся ключи
pub struct OrderedJson(pub IndexMap<String, Value>);

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

pub fn pack_tbl_into_obj_in_arr(rows: Vec<Row>) -> Result<Vec<OrderedJson>, Error> {
    rows.into_iter()
        .map(|row| {
            let mut hmap = OrderedJson::new();

            for column in row.columns().iter() {
                let k = column.name().to_string();
                let v = convert_to_serde_json_type(&row, column)?;

                hmap.insert(k, v);
            }
            Ok(hmap) // Обратите внимание, что здесь мы возвращаем Ok(hmap)
        })
        .collect() // но collect() автоматически соберет значения в Result<Vec<OrderedJson>, Error>
}

pub fn pack_tbl_into_arr_in_obj(rows: Vec<Row>) -> Result<IndexMap<String, Value>, Error> {
    let mut hmap = IndexMap::new();

    for row in &rows {
        for column in row.columns().iter() {
            let value = hmap
                .entry(column.name().to_string())
                .or_insert_with(|| Value::Array(Vec::with_capacity(rows.len())));

            let v = convert_to_serde_json_type(row, column)?;

            match value {
                Value::Array(arr) => {
                    arr.push(v);
                }
                _ => {
                    return Err(Error::InternalLogic(
                        "значение IndexMap не serde_json::value::Array".to_string(),
                    ));
                }
            }
        }
    }

    Ok(hmap)
}

pub fn convert_to_serde_json_type(row: &Row, column: &Column) -> Result<Value, Error> {
    //потенциально добавить: pg_lsn
    Ok(match *column.type_() {
        Type::BOOL => match row.try_get::<_, Option<bool>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::CHAR => match row.try_get::<_, Option<i8>>(column.name()) {
            Ok(Some(v)) => {
                let ch = char::from_u32(v as u32).ok_or_else(|| {
                    Error::InternalLogic(
                        "невозможное условие при конвертировании типа u32 в char".to_string(),
                    )
                })?;
                json!(ch.to_string())
            }
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::INT2 => match row.try_get::<_, Option<i16>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::INT4 => match row.try_get::<_, Option<i32>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::OID => match row.try_get::<_, Option<u32>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::INT8 => match row.try_get::<_, Option<i64>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::FLOAT4 => match row.try_get::<_, Option<f32>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::FLOAT8 => match row.try_get::<_, Option<f64>>(column.name()) {
            Ok(Some(v)) => json!(v),
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        Type::DATE => match row.try_get::<_, Option<NaiveDate>>(column.name()) {
            Ok(Some(v)) => {
                // let base_date = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                // let duration = v.signed_duration_since(base_date);
                // json!(duration.num_days())
                json!(v.format("%Y-%m-%d").to_string())
            }
            Ok(None) => Value::Null,
            Err(err) => return Err(Error::DataRetrieval(err)),
        },
        _ => match row.try_get::<_, Option<String>>(column.name()) {
            // VARCHAR, CHAR(n), TEXT, CITEXT, NAME
            Ok(v) => json!(v),
            Err(_) => Value::Null,
        },
    })
}