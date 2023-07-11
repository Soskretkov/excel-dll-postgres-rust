use std::slice;
// структура с данными по которым vba получит строку из памяти, #[repr(C)] зафиксирует поля в порядке как задал программист
#[repr(C)]
pub struct StringWrapForVBA {
    ptr: *mut u16,
    length_in_bytes: i32,
    _data: Box<Vec<u16>>, // это поле не будет читаться VBA и оно тут для выравнивания времени жизни с полем "ptr" для того чтобы vba читал действительный "ptr"
}

#[no_mangle]
pub extern "stdcall" fn send_request(bstr_sql_code_ptr: *const u16) -> *mut StringWrapForVBA {
    // вычисление длины строки

    // let len_bstr_ptr = unsafe { bstr_sql_code.offset(-4) as *const u32 };

    // let len_bstr: u32 = unsafe {
    //     *len_bstr_ptr
    //     //std::ptr::read_unaligned(len_ptr) as usize
    // };






    // конвертация из *const u16 в String
    // let sql_code = unsafe {
    //     let slice = slice::from_raw_parts(bstr_sql_code, 2); // создание среза
    //     String::from_utf16(slice).unwrap() // конвертация в String
    // };

    // let text = getDatabaseResponse(sql_code);
    // let text = len_bstr.to_string();




    // let text = format!("{:p}", bstr_sql_code_ptr);
    let text = format!("rust-адрес: {}", bstr_sql_code_ptr as usize);





    // let text = sql_code;
    let sending_data = get_string_wrap_for_vba(text);

    Box::into_raw(Box::new(sending_data))
}

fn get_string_wrap_for_vba(text: String) -> StringWrapForVBA {
    //.encode_utf16() правильно обрабатывает суррогатные пары Unicode (возвращает итератор 16-битных юнитов кодировки UTF-16).
    let data: Vec<u16> = text.encode_utf16().collect();
    let length_in_bytes: i32 = (text.encode_utf16().count() * std::mem::size_of::<u16>())
        .try_into()
        .unwrap();

    let boxed_data = Box::new(data);
    let ptr = boxed_data.as_ptr() as *mut u16;

    StringWrapForVBA {
        ptr,
        length_in_bytes,
        _data: boxed_data,
    }
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut StringWrapForVBA) {
    unsafe {
        // освобождаем память под структурой
        drop(Box::from_raw(ptr));
    }
}

fn getDatabaseResponse(query: String) -> String {
    let database_response = "ответ𐐷".to_string();
    database_response
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}