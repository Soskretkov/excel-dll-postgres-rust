use std::os::raw::c_void;

// структура с данными по которым vba считает строку из памяти, #[repr(C)] зафиксирует поля в порядке как задал программист
#[repr(C)]
pub struct SendingStringForVBA {
    ptr: *mut c_void,
    length: i32,
}

#[no_mangle]
pub extern "stdcall" fn send_request() -> *mut String {
    let text = getDatabaseResponse();
    let text_ptr = Box::new(text);

    Box::into_raw(text_ptr)
}

#[no_mangle]
pub extern "stdcall" fn get_string(ptr: *mut String) -> Box<SendingStringForVBA> {
    let mut data: Vec<u16> = unsafe { (*ptr).encode_utf16().collect() };
    free_data(ptr);
    data.push(0); // null terminate

    let sending_data = Box::new(SendingStringForVBA {
        ptr: data.as_mut_ptr() as *mut c_void,
        length: (data.len() * std::mem::size_of::<u16>())
            .try_into()
            .unwrap(),
    });

    sending_data
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut String) {
    unsafe {
        // Освобождение памяти, на которую указывает ptr
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
