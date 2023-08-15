use std::collections::HashMap;
use std::str::FromStr;
mod vba_str_io;
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::ser::{SerializeMap, Serializer};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::runtime;
use tokio_postgres::{types::Type, Column, NoTls, Row};
use vba_str_io::StringForVba;
mod error;
use error::Error;

#[derive(Deserialize)]
struct ApiRequest {
    sql_query: String,
    requesters_id: Option<String>, //имя листа или таблицы
    is_obj_in_arr_response: bool,
}

impl FromStr for ApiRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(Error::JsonDeserialization)
    }
}

enum ResponseType {
    ArrInObj(OrderedJson),
    ObjInArr(Vec<OrderedJson>),
}

impl Serialize for ResponseType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseType::ArrInObj(value) => value.serialize(serializer),
            ResponseType::ObjInArr(value) => value.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
struct ApiResponse {
    data: Result<ResponseType, Error>,
    requesters_id: Option<String>,
}

// сиротское правило не дает реализовать трейт Serialize непосредственно на IndexMap
struct OrderedJson(IndexMap<String, Value>);

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

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVba {
    let wraped_responses_vec: Result<_, Error> = {
        || {
            let string_from_vba =
                vba_str_io::get_string_from_vba(ptr).map_err(Error::InvalidUtf16OnInput)?;

            let excel_requests: Vec<ApiRequest> =
                serde_json::from_str(&string_from_vba).map_err(Error::JsonDeserialization)?;

            let my_db_params = get_db_params(); // параметры для подключения к БД

            let tokio_rows_vec = get_database_response(&excel_requests, my_db_params)?; // ответ БД

            let responses_vec = map_rows_to_api_responses_vec(excel_requests, tokio_rows_vec);

            Ok(responses_vec)
        }
    }();

    // сериализация и собственная ошибка на случай провала serde_json
    let sent_json_txt = serde_json::to_string(&wraped_responses_vec)
        .map_err(Error::JsonSerialization)
        .unwrap_or_else(|err| {
            serde_json::json!(Err::<Vec<ApiResponse>, error::Error>(err)).to_string()
        });

    //тест
    // let forced_error = Error::JsonSerialization(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "forced serialization error")));
    // let sent_json_txt = serde_json::to_string(&Result::<Vec<ApiResponse>, Error>::Err(forced_error))
    //     .unwrap_or_else(|err| serde_json::json!(Err::<Vec<ApiResponse>, error::Error>(Error::JsonSerialization(err))).to_string());
    // let sent_json_txt =  serde_json::json!({"Err":{"code":1004,"descr":"не удалось сериализовать ответ БД в JSON","tech_descr":""}}).to_string(); //тест 2

    // конвертация в формат, ожидаемый на стороне vba
    let string_for_vba = StringForVba::from_string(sent_json_txt);
    string_for_vba.into_raw()
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_data(ptr: *mut StringForVba) {
    drop(Box::from_raw(ptr)); // освобождаем память
}

//для вызова из кода на других языках, используется соглашение о вызове stdcall (обычно используемое в Windows для вызовов функций API)
fn get_database_response(
    excel_requests: &[ApiRequest],
    db_access_parameters: HashMap<String, String>,
) -> Result<Vec<Result<Vec<Row>, Error>>, Error> {
    // строка параметров для соединения с БД
    let parameter_string = format!(
        "host={} dbname={} user={} password={}",
        db_access_parameters.get("host").unwrap(),
        db_access_parameters.get("dbname").unwrap(),
        db_access_parameters.get("user").unwrap(),
        db_access_parameters.get("password").unwrap()
    );

    // Tokio автоматически создает рантайм для асинхронных операций, но ниже это делается вручную - код не в асинхронной среде
    let rt = runtime::Runtime::new().map_err(Error::TokioRuntimeCreation)?;

    // NoTls - не требуетя защищенного соединения, что приемлемо в защищенной среде
    let (client, connection) = rt
        .block_on(tokio_postgres::connect(&parameter_string, NoTls))
        .map_err(Error::DBConnection)?;

    // запускает асинхронную задачу, которая ожидает завершения соединения с БД. Если ошибка, она будет записана в стандартный поток ошибок
    rt.spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut res: Vec<Result<Vec<Row>, Error>> = Vec::new();
    for request in excel_requests {
        res.push(
            rt.block_on(client.query(&request.sql_query, &[]))
                .map_err(Error::SqlExecution),
        );
    }

    Ok(res)
}

fn map_rows_to_api_responses_vec(
    excel_requests: Vec<ApiRequest>,
    data_vec: Vec<Result<Vec<Row>, Error>>,
) -> Vec<ApiResponse> {
    let mut res = Vec::with_capacity(excel_requests.len());

    for (request, rows_vec) in excel_requests.into_iter().zip(data_vec) {
        let data = rows_vec.and_then(|rows| match request.is_obj_in_arr_response {
            true => {
                let res = rows
                    .into_iter()
                    .map(|row| {
                        let mut hmap: OrderedJson = OrderedJson::new();

                        for column in row.columns().iter() {
                            let k = column.name().to_string();
                            let v = convert_to_serde_json_type(&row, column);

                            hmap.insert(k, v);
                        }
                        hmap
                    })
                    .collect();

                Ok(ResponseType::ObjInArr(res))
            }

            false => unimplemented!(),
        });

        let excel_response = ApiResponse {
            data,
            requesters_id: request.requesters_id,
        };

        res.push(excel_response);
    }
    res
}

fn convert_to_serde_json_type(row: &Row, column: &Column) -> Value {
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
