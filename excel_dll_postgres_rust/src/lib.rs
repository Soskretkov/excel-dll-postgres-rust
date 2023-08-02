use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
mod vba_str_io;
use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::ser::{SerializeMap, Serializer};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tokio;
use tokio_postgres::{types::Type, NoTls, Row};
use vba_str_io::StringForVBA;

#[derive(Deserialize)]
struct ExcelRequest {
    sql_query: String,
    requesters_id: Option<String>, //имя листа или таблицы
}

impl FromStr for ExcelRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s).map_err(Error::JsonDeserialization)?)
    }
}

#[derive(Serialize)]
struct ExcelResponse {
    data: Result<Vec<OrderedJson>, Error>,
    requesters_id: Option<String>,
}

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

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    let wraped_responses_vec: Result<_, Error> = {
        || {
            let string_from_vba =
                vba_str_io::get_string_from_vba(ptr).map_err(|_| Error::InvalidUTF16FromVba)?;

            let excel_requests: Vec<ExcelRequest> =
                serde_json::from_str(&string_from_vba).map_err(Error::JsonDeserialization)?;

            let my_db_params = get_db_params(); // параметры для подключения к БД

            let tokio_rows_vec = get_database_response(&excel_requests, my_db_params)?; // ответ БД

            let responses_vec = rows_type_into_obj_in_arr_json(excel_requests, tokio_rows_vec);

            Ok(responses_vec)
        }
    }();

    // let wraped_sent_json_txt =
    //     serde_json::to_string(&wraped_responses_vec).map_err(Error::JsonSerialization);
    // let (sent_json_txt, is_valid) = match wraped_sent_json_txt {
    //     Ok(sent_json_txt) => (sent_json_txt, true),
    //     Err(err) => (format!("{}", err), false),
    // };

    // сериализация и текст json-ошибки для excel на случай провала
    let sent_json_txt = serde_json::to_string(&wraped_responses_vec)
        .map_err(Error::JsonSerialization)
        .unwrap_or_else(|err| serde_json::json!({ "Err": err.to_string() }).to_string());

    // тест
    // let sent_json_txt =  serde_json::json!({ "Err": format!("не удалось сериализовать ответ БД в JSON-формат: {}", "ОШИБКА") }).to_string();

    // конвертация в формат, ожидаемый на стороне vba
    let string_for_vba = StringForVBA::from_string(sent_json_txt);
    // string_for_vba.validity_update(is_valid);
    string_for_vba.into_raw()
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut StringForVBA) {
    unsafe {
        // освобождаем память под структурой
        drop(Box::from_raw(ptr));
    }
}

//для вызова из кода на других языках, используется соглашение о вызове stdcall (обычно используемое в Windows для вызовов функций API)
fn get_database_response(
    excel_requests: &[ExcelRequest],
    db_access_parameters: HashMap<String, String>,
) -> Result<Vec<Result<Vec<Row>, Error>>, Error> {
    // ) -> Option<Result<Vec<Row>, Error>> {
    // строка параметров для соединения с БД
    let parameter_string = format!(
        "host={} dbname={} user={} password={}",
        db_access_parameters.get("host").unwrap(),
        db_access_parameters.get("dbname").unwrap(),
        db_access_parameters.get("user").unwrap(),
        db_access_parameters.get("password").unwrap()
    );

    // Tokio автоматически создает рантайм для асинхронных операций, но ниже это делается вручную - код не в асинхронной среде
    let rt = tokio::runtime::Runtime::new().map_err(Error::TokioRuntimeCreation)?;

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

fn rows_type_into_obj_in_arr_json(
    excel_requests: Vec<ExcelRequest>,
    data_vec: Vec<Result<Vec<Row>, Error>>,
) -> Vec<ExcelResponse> {
    let mut res = Vec::with_capacity(excel_requests.len());
    for (request, rows_vec) in excel_requests.into_iter().zip(data_vec) {
        let data = rows_vec.map(|rows| {
            rows.into_iter()
                .map(|row| {
                    let mut hmap: OrderedJson = OrderedJson::new();

                    for (i, column) in row.columns().into_iter().enumerate() {
                        let k = column.name().to_string();
                        let v: serde_json::Value = match *column.type_() {
                            Type::BOOL => match row.try_get::<_, bool>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::CHAR => match row.try_get::<_, i8>(i) {
                                Ok(v) => {
                                    let ch = char::from_u32(v as u32).unwrap_or('\0');
                                    serde_json::json!(ch.to_string())
                                }
                                Err(_) => serde_json::json!(null),
                            },
                            Type::INT2 => match row.try_get::<_, i16>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::INT4 => match row.try_get::<_, i32>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::OID => match row.try_get::<_, u32>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::INT8 => match row.try_get::<_, i64>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::FLOAT4 => match row.try_get::<_, f32>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::FLOAT8 => match row.try_get::<_, f64>(i) {
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                            Type::DATE => match row.try_get::<_, NaiveDate>(i) {
                                Ok(v) => {
                                    // let base_date = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                                    // let duration = v.signed_duration_since(base_date);
                                    // serde_json::json!(duration.num_days())
                                    serde_json::json!(v.format("%Y-%m-%d").to_string())
                                }
                                Err(_) => serde_json::json!(null),
                            },
                            _ => match row.try_get::<_, String>(i) {
                                //VARCHAR, CHAR(n), TEXT, CITEXT, NAME
                                Ok(v) => serde_json::json!(v),
                                Err(_) => serde_json::json!(null),
                            },
                        };
                        hmap.insert(k, v);
                    }
                    hmap
                })
                .collect()
        });

        let excel_response = ExcelResponse {
            data,
            requesters_id: request.requesters_id,
        };

        res.push(excel_response);
    }

    res
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

#[derive(Debug)] #[rustfmt::skip]
enum Error {
    InvalidUTF16FromVba,
    DBConnection(tokio_postgres::Error),
    SqlExecution(tokio_postgres::Error),
    TokioRuntimeCreation(std::io::Error),
    JsonSerialization(serde_json::Error),
    JsonDeserialization(serde_json::Error),
}

#[rustfmt::skip]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DBConnection(err) => write!(
                f,"{}: {}", "01",
                format!("не удалось подключение к базе данных: {}", err)
            ),
            Error::SqlExecution(err) => write!(
                f,"{}: {}", "02",
                format!("не удалось выполнить SQL-запрос: {}", err)
            ),
            Error::JsonDeserialization(err) => write!(
                f,"{}: {}", "03",
                format!("не валидные аргументы переданы из vba-кода макроса Excel в dll: {}", err)
            ),
            Error::InvalidUTF16FromVba => write!(
                f,"{}: {}", "10", "не удалось конвертировать запрос в UTF-16"
            ),
            Error::JsonSerialization(err) => write!(
                f,"{}: {}", "11",
                format!("не удалось сериализовать ответ БД в JSON-формат: {}", err)
            ),
            Error::TokioRuntimeCreation(err) => write!(
                f,"{}: {}", "12",
                format!("не удалось создать рантайм Tokio: {}", err)
            ),
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let error_str = self.to_string();
        serializer.serialize_str(&error_str)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
