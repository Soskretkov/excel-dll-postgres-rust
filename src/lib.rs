mod vba_string_utils;
use vba_string_utils::StringForVBA;

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    // let sql_query = vba_string_utils::get_string_from_vba(ptr);
    let sql_query = String::from("select * from ref_currency_type;");

    let response = getDatabaseResponse(sql_query);
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

fn getDatabaseResponse(query: String) -> String {
    let database_response = format!("запрос: {}\n\nбайты: {}", query, getBytes(&query));
    database_response
}

fn getBytes(text: &str) -> String {
    let bytes = text.as_bytes();
    let hex_string = bytes
        .iter()
        .map(|&byte| format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .join("");
    hex_string
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
