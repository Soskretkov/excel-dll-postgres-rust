mod vba_str_io;
use vba_str_io::StringForVBA;

use postgres::{Client, Error, NoTls, Row};

#[no_mangle]
pub extern "stdcall" fn send_request(ptr: *const u16) -> *mut StringForVBA {
    let sql_query = String::from("select * from ref_currency_type;");

    let response = getDatabaseResponse(&sql_query).unwrap();
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

fn getDatabaseResponse(query: &str) -> Result<String, Error> {

    // Создаем клиент и подключаемся к базе данных
    let mut client = Client::connect("host=localhost user=postgres dbname=el_dabaa", NoTls)?;

    // Выполняем запрос
    let rows: Vec<Row> = client.query(query, &[])?;
    let response: String = rows[0].get("id");

    // Преобразуем результаты запроса в JSON
    // let json = convert_to_json(rows);

    // Ok(value)
    Ok(response)
}

fn convert_to_json() {}











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
