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
    #[serde(rename = "sqlQuery")]
    pub sql_query: String,
    #[serde(rename = "isObjInArrFmt")]
    pub is_obj_in_arr_fmt: bool,
}

impl FromStr for ApiRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(Error::Deserialization)
    }
}

pub enum SqlResponseTable {
    ArrInObj(OrderedJson),
    ObjInArr(Vec<OrderedJson>),
}

impl Serialize for SqlResponseTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SqlResponseTable::ArrInObj(value) => value.serialize(serializer),
            SqlResponseTable::ObjInArr(value) => value.serialize(serializer),
        }
    }
}

pub fn map_rows_to_api_responses_vec(
    excel_requests: Vec<ApiRequest>,
    data_vec: Vec<Result<Vec<Row>, Error>>,
) -> Result<Vec<Result<SqlResponseTable, Error>>, Error> {
    let mut res = Vec::with_capacity(excel_requests.len());

    for (request, rows_vec) in excel_requests.into_iter().zip(data_vec) {
        let data = rows_vec.and_then(|rows| match request.is_obj_in_arr_fmt {
            true => {
                let pack_tbl = json_utils::pack_tbl_into_obj_in_arr(rows);
                pack_tbl.map(|vec| SqlResponseTable::ObjInArr(vec))
            }
            false => {
                let pack_tbl = json_utils::pack_tbl_into_arr_in_obj(rows);
                pack_tbl.map(|index_map| SqlResponseTable::ArrInObj(OrderedJson(index_map)))
            }
        });

        res.push(data);
    }
    Ok(res)
}
