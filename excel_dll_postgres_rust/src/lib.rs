use std::collections::HashMap;
mod vba_str_io;
use tokio;
use tokio_postgres::{types::Type, NoTls, Row};
use vba_str_io::StringForVBA;

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    let sql_query = vba_str_io::get_string_from_vba(ptr);

    // параметры для подключения к БД
    let my_db_parameters = get_db_params();

    // ответ от БД
    let response = get_database_response(&sql_query, my_db_parameters);

    // конвертация в формат, ожидаемый на стороне vba
    let response_for_vba = StringForVBA::from_string(response);
    response_for_vba.into_raw()
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut StringForVBA) {
    unsafe {
        // освобождаем память под структурой
        drop(Box::from_raw(ptr));
    }
}

//для вызова из кода на других языках, используется соглашение о вызове stdcall (обычно используемое в Windows для вызовов функций API)
fn get_database_response(query: &str, db_access_parameters: HashMap<String, String>) -> String {
    // строка параметров для соединения с БД
    let parameter_string = format!(
        "host={} dbname={} user={} password={}",
        db_access_parameters.get("host").unwrap(),
        db_access_parameters.get("dbname").unwrap(),
        db_access_parameters.get("user").unwrap(),
        db_access_parameters.get("password").unwrap()
    );

    // Tokio автоматически создает рантайм для асинхронных операций и не нужно его создавать (как тут), однако наш код не в асинхронной среде
    let rt = tokio::runtime::Runtime::new().unwrap();

    // NoTls - не требуетя защищенного соединения, что приемлемо в защищенной среде
    let (client, connection) = rt
        .block_on(tokio_postgres::connect(&parameter_string, NoTls))
        .unwrap();

    rt.spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows: Vec<Row> = rt.block_on(client.query(query, &[])).unwrap();

    // результаты запроса в String c json-контентом
    let json = rows_type_into_obj_in_arr_json(rows);

    json
}

fn rows_type_into_obj_in_arr_json(rows: Vec<Row>) -> String {
    use chrono::NaiveDate;
    use indexmap::IndexMap;
    use serde::ser::{SerializeMap, Serializer};
    use serde::Serialize;
    use serde_json::Value;

    // сиротское правило не дает реализовать трейт Serialize непосредственно на IndexMap
    struct OrderedJson(IndexMap<String, Value>);

    impl Serialize for OrderedJson {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let OrderedJson(ref inner_map) = *self;
            let mut map = serializer.serialize_map(Some(inner_map.len()))?;
            for (k, v) in inner_map {
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

    let results: Vec<_> = rows
        .into_iter()
        .map(|row| {
            let mut hmap: OrderedJson = OrderedJson::new();

            for (i, column) in row.columns().into_iter().enumerate() {
                let k = column.name().to_string();
                let v: serde_json::Value = match *column.type_() {
                    Type::BOOL => {
                        match row.try_get::<_, bool>(i) {
                            Ok(v) => serde_json::json!(v),
                            Err(_) => serde_json::json!(null),
                        }
                    }
                    Type::INT2 => {
                        let v: Result<i16, _> = row.try_get(i);
                        serde_json::json!(v.unwrap())
                    }
                    Type::INT4 => {
                        let v: Result<i32, _> = row.try_get(i);
                        serde_json::json!(v.unwrap())
                    }
                    Type::INT8 => {
                        let v: Result<i64, _> = row.try_get(i);
                        serde_json::json!(v.unwrap())
                    }
                    Type::FLOAT4 => {
                        let v: Result<f32, _> = row.try_get(i);
                        serde_json::json!(v.unwrap())
                    }
                    Type::FLOAT8 => {
                        let v: Result<f64, _> = row.try_get(i);
                        serde_json::json!(v.unwrap())
                    }
                    Type::DATE => {
                        let v: Result<NaiveDate, _> = row.try_get(i);
                        let base_date = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                        match v {
                            Ok(v) => {
                                let duration = v.signed_duration_since(base_date);
                                serde_json::json!(duration.num_days())
                            }
                            Err(_) => serde_json::json!(null),
                        }
                    }
                    _ => {
                        let v: Result<String, _> = row.try_get(i);
                        serde_json::json!(v.unwrap_or_else(|_| "".to_string()))
                    }
                };
                hmap.insert(k, v);
            }

            hmap
        })
        .collect();

    serde_json::to_string(&results).unwrap()
}

fn get_db_params() -> HashMap<String, String> {
    // это только для теста, не храните реальные данные в dll!
    let mut db_parameters = HashMap::<String, String>::new();
    db_parameters.insert(String::from("host"), String::from("localhost"));
    db_parameters.insert(String::from("dbname"), String::from("el_dabaa"));
    db_parameters.insert(String::from("user"), String::from("postgres"));
    db_parameters.insert(String::from("password"), String::from("''"));
    db_parameters
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}

// fn rows_type_into_obj_in_arr_json(rows: Vec<Row>) -> String {
//     use chrono::NaiveDate;
//     use indexmap::IndexMap; //чтобы сохранить порядок столбцов
//     use serde::ser::{SerializeMap, Serializer};
//     use serde::Serialize;
//     use serde_json::Value;

//     // сиротское правило не дает реализовать трейт Serialize непосредственно на IndexMap
//     struct OrderedJson(IndexMap<String, Value>);

//     impl Serialize for OrderedJson {
//         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//         {
//             let OrderedJson(ref inner_map) = *self;
//             let mut map = serializer.serialize_map(Some(inner_map.len()))?;
//             for (k, v) in inner_map {
//                 map.serialize_entry(k, v)?;
//             }
//             map.end()
//         }
//     }

//     impl OrderedJson {
//         pub fn new() -> Self {
//             OrderedJson(IndexMap::new())
//         }
//         pub fn insert(&mut self, key: String, value: Value) {
//             self.0.insert(key, value);
//         }
//     }

//     let results: Vec<_> = rows
//         .into_iter()
//         .map(|row| {
//             let mut hmap: OrderedJson = OrderedJson::new();

//             for column in row.columns() {
//                 let k = column.name().to_string();
//                 match column.type_() {
//                     &Type::BOOL => {
//                         let v: bool = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::INT2 => {
//                         let v: i16 = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::INT4 => {
//                         let v: i32 = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::INT8 => {
//                         let v: i64 = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::FLOAT4 => {
//                         let v: f32 = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::FLOAT8 => {
//                         let v: f64 = row.get(k.as_str());
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                     &Type::DATE => {
//                         let v: Option<NaiveDate> = row.get(k.as_str());
//                         let base_date = NaiveDate::from_ymd_opt(1899, 12, 30);
//                         if let Some(base_date) = base_date {
//                             if let Some(v) = v {
//                                 let duration = v.signed_duration_since(base_date);
//                                 hmap.insert(k, serde_json::json!(duration.num_days()));
//                             } else {
//                                 // Handle null value:
//                                 hmap.insert(k, serde_json::json!(null));
//                             }
//                         } else {
//                             // Handle error:
//                             println!("Invalid base date");
//                         }
//                     }
//                     &Type::JSONB => {
//                     }
//                     _ => {
//                         let value: Result<String, _> = row.try_get(k.as_str());
//                         let v = match value {
//                             Ok(v) => v,
//                             Err(_) => "".to_string(),
//                         };
//                         hmap.insert(k, serde_json::json!(v));
//                     }
//                 }
//             }
//             hmap
//         })
//         .collect();

//     serde_json::to_string(&results).unwrap()
// }
