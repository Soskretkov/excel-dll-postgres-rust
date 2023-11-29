// Назначение модуля кратко: работа с API-запросами и ответами.
// Подробное описание: модуль предназначен для обработки входящих API-запросов и формирования соответствующих
// ответов. Определяет структуры запросов и ответов и содержит логику, которая связана с обработкой
// этих запросов и формированием ответов. Модуль является связующим звеном внешним API и внутренней
// логикой приложения.
use super::json_utils;
use super::Error;
use json_utils::OrderedJson;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio_postgres::Row;

#[derive(Deserialize)]
pub struct ApiRequest {
    pub requesters_id: Option<String>, //имя листа или таблицы
    pub sql_query: String,
    pub is_obj_in_arr_fmt: bool,
}

impl FromStr for ApiRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(Error::JsonDeserialization)
    }
}

pub enum ResponseType {
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
pub struct ApiResponse {
    pub requesters_id: Option<String>,
    pub data: Result<ResponseType, Error>,
}

pub fn map_rows_to_api_responses_vec(
    excel_requests: Vec<ApiRequest>,
    data_vec: Vec<Result<Vec<Row>, Error>>,
) -> Result<Vec<ApiResponse>, Error> {
    let mut res = Vec::with_capacity(excel_requests.len());

    for (request, rows_vec) in excel_requests.into_iter().zip(data_vec) {
        let data = rows_vec.and_then(|rows| match request.is_obj_in_arr_fmt {
            true => {
                let pack_tbl = json_utils::pack_tbl_into_obj_in_arr(rows);
                // любые ошибки пакуем в ApiResponse, не прерывая обработку остальных запросов
                pack_tbl.and_then(|vec| Ok(ResponseType::ObjInArr(vec)))
            }
            false => {
                let pack_tbl = json_utils::pack_tbl_into_arr_in_obj(rows);
                pack_tbl.and_then(|index_map| Ok(ResponseType::ArrInObj(OrderedJson(index_map))))
            }
        });

        let excel_response = ApiResponse {
            data,
            requesters_id: request.requesters_id,
        };

        res.push(excel_response);
    }
    Ok(res)
}
