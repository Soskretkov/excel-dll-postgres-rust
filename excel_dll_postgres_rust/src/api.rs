use serde::{Deserialize, Serialize};
use std::str::FromStr;
use serde::ser::Serializer;
use tokio_postgres::Row;
use super::Error;
use super::json_utils;
use json_utils::OrderedJson;

#[derive(Deserialize)]
pub struct ApiRequest {
    pub sql_query: String,
    pub requesters_id: Option<String>, //имя листа или таблицы
    pub is_obj_in_arr_response: bool,
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
    pub data: Result<ResponseType, Error>,
    pub requesters_id: Option<String>,
}

pub fn map_rows_to_api_responses_vec(
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
                            let v = json_utils::convert_to_serde_json_type(&row, column);

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