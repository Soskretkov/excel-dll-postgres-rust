mod vba_str_io;
mod api;
mod json_utils;
mod db;
mod error;
use vba_str_io::StringForVba;
use error::Error as Error;
use api::{ApiRequest, ResponseType};

//для вызова из кода на других языках, используется соглашение о вызове stdcall (обычно используемое в Windows для вызовов функций API)
#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVba {
    let wraped_responses_vec: Result<_, Error> = {
        || {
            let string_from_vba =
                vba_str_io::get_string_from_vba(ptr).map_err(Error::InvalidUtf16OnInput)?;

            let excel_requests: Vec<ApiRequest> =
                serde_json::from_str(&string_from_vba).map_err(Error::JsonDeserialization)?;

            let my_db_params = db::get_db_auth_data(); // параметры для подключения к БД

            let tokio_rows_vec = db::get_database_response(&excel_requests, my_db_params)?; // ответ БД

            let responses_vec = api::map_rows_to_api_responses_vec(excel_requests, tokio_rows_vec)?;

            Ok(responses_vec)
        }
    }();

    // сериализация и собственная ошибка на случай провала serde_json
    let sent_json_txt = serde_json::to_string(&wraped_responses_vec)
        .map_err(Error::JsonSerialization)
        .unwrap_or_else(|err| {
            serde_json::json!(Err::<Vec<Result<ResponseType, Error>>, Error>(err)).to_string()
        });

    // тест
    // let forced_error = Error::JsonSerialization(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "forced serialization error")));
    // let sent_json_txt = serde_json::to_string(&Result::<Vec<Result<ResponseType, Error>>, Error>::Err(forced_error))
    //     .unwrap_or_else(|err| serde_json::json!(Err::<Vec<Result<ResponseType, Error>>, Error>(Error::JsonSerialization(err))).to_string());

    // конвертация в формат, ожидаемый на стороне vba
    let string_for_vba = StringForVba::from_string(sent_json_txt);
    string_for_vba.into_raw()
}

#[no_mangle]
pub unsafe extern "stdcall" fn free_data(ptr: *mut StringForVba) {
    drop(Box::from_raw(ptr)); // освобождаем память
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}