// структура с данными по которым vba считает строку из памяти, #[repr(C)] зафиксирует поля в порядке как задал программист
//Если есть только Box<Vec<u16>> без Option, не сможете использовать std::mem::forget, поскольку Box<Vec<u16>> всегда должен иметь действительное значение. Option<Box<Vec<u16>>> позволяет использовать None для обозначения отсутствия значения, что позволяет безопасно использовать std::mem::forget.
#[repr(C)]
pub struct SendingStringForVBA {
    ptr: *mut u16,
    length_in_bytes: i32,
    _data: Option<Box<Vec<u16>>>, // это поле будет читаться VBA
}

#[no_mangle]
pub extern "stdcall" fn send_request() -> *mut SendingStringForVBA {
    let text = getDatabaseResponse();
    let sending_data = get_string(text);

    Box::into_raw(Box::new(sending_data))
}

// Преобразует String в SendingStringForVBA
fn get_string(text: String) -> SendingStringForVBA {
    //.encode_utf16() правильно обрабатывает суррогатные пары Unicode (возвращает итератор, который выдает 16-битные юниты кодировки UTF-16).
    //тогда как chars().count() возвращает количество Unicode скаляров в строке, не все из которых будут соответствовать одному символу в строке
    let data: Vec<u16> = text.encode_utf16().collect();
    let length_in_bytes: i32 = (text.encode_utf16().count() * std::mem::size_of::<u16>()).try_into().unwrap();

    let boxed_data = Box::new(data);
    let ptr = boxed_data.as_ptr() as *mut u16;

    SendingStringForVBA {
        ptr,
        length_in_bytes,
        _data: Some(boxed_data),
    }
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut SendingStringForVBA) {
    unsafe {
        // освобождаем память под структурой
        drop(Box::from_raw(ptr));
    }
}

fn getDatabaseResponse() -> String {
    let database_response = "ответ𐐷".to_string();
    database_response
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}