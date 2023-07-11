mod vba_string_utils;
use vba_string_utils as vsu;

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut vsu::StringWrapForVBA {
    let sql_query = vsu::get_string_from_vba(ptr);

    let response = getDatabaseResponse(sql_query);

    vsu::get_string_ptr_for_vba(response)
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut vsu::StringWrapForVBA) {
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