mod vba_str_io;
use vba_str_io::StringForVBA;

use std::collections::HashMap;
use postgres::{Client, NoTls, Row, types::Type};
use serde_json::Value;

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    let sql_query = String::from("select * from ref_currency_type;");

    let response = getDatabaseResponse(&sql_query);
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

fn getDatabaseResponse(query: &str) -> String {
    // cоздаем клиент и подключаемся к базе данных
    let mut client = Client::connect("host=localhost user=postgres dbname=el_dabaa", NoTls).unwrap();

    // выполняем запрос
    let rows: Vec<Row> = client.query(query, &[]).unwrap();

    // результаты запроса в формат, понятный serde_json
    let results:Vec<_> = rows
        .into_iter()
        .map(|row| {
            let mut hmap: HashMap<String, Value> = HashMap::new();
            for column in row.columns() {
                let k = column.name().to_string();
                match column.type_() {
                    &Type::BOOL => {
                        let v: bool = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));                   
                    }
                    &Type::INT2 => {
                        let v: i16 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    &Type::INT4 => {
                        let v: i32 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    &Type::INT8 => {
                        let v: i64 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    &Type::FLOAT4 => {
                        let v: f32 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    &Type::FLOAT8 => {
                        let v: f64 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    &Type::DATE => {
                        let v: i32 = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                    _ => {
                        let v: String = row.get(k.as_str());
                        hmap.insert(k, serde_json::json!(v));
                    }
                }
            }
            hmap
        })
        .collect();

    let json = serde_json::to_string(&results).unwrap();

    json
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}