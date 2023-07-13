mod vba_str_io;
use vba_str_io::StringForVBA;
use postgres::{Client, NoTls, Row, types::Type};

use serde_json::Value;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use indexmap::IndexMap;


#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    let sql_query = vba_str_io::get_string_from_vba(ptr);
    // let sql_query = "SELECT * FROM ref_currency_type;";

    let response = getDatabaseResponse(&sql_query);
    let response_for_vba = StringForVBA::from_string(response);
    // let response_for_vba = StringForVBA::from_string("a".to_string());

    response_for_vba.into_raw()
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut StringForVBA) {
    unsafe {
        // освобождаем память под структурой
        drop(Box::from_raw(ptr));
    }
}

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

fn getDatabaseResponse(query: &str) -> String {
    // cоздаем клиент и подключаемся к базе данных
    let mut client = Client::connect("host=localhost user=postgres dbname=el_dabaa", NoTls).unwrap();

    // выполняем запрос
    let rows: Vec<Row> = client.query(query, &[]).unwrap();

    // результаты запроса в формат, понятный serde_json
    let results:Vec<_> = rows
        .into_iter()
        .map(|row| {
            let mut hmap: OrderedJson = OrderedJson::new();

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
                    //     let v: i32 = row.get(k.as_str());
                    //     hmap.insert(k, serde_json::json!(v));
                    }
                    _ => {
                        let value: Result<String, _> = row.try_get(k.as_str());
                        let v = match value {
                            Ok(v) => v,
                            Err(_) => "".to_string(),
                        };
                        hmap.insert(k, serde_json::json!(v));
                    }
                }
            }
            hmap
        })
        .collect();

    // let results = {
    //         let mut hmap: HashMap<String, Value> = HashMap::new();
    //         for column in rows[0].columns() {
    //             let k = column.name().to_string();
    //             let v: String = column.type_().to_string();
    //             hmap.insert(k, serde_json::json!(v));
    //         }
    //         hmap
    //     };

    let json = serde_json::to_string(&results).unwrap();

    json
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}