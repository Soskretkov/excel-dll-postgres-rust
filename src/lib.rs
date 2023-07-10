use std::collections::VecDeque;
use std::os::raw::c_void;

struct DbResponse {
    ptr: *mut c_void,
    length: usize,
}

static mut POSTGRES_RESPONSE: Option<DbResponse> = None;

static mut STRINGS: VecDeque<Vec<u16>> = VecDeque::new();

#[no_mangle]
pub extern "stdcall" fn send_request() {
    let response_text = getDatabaseResponse();
    let mut data: Vec<u16> = response_text.encode_utf16().collect();
    data.push(0); // null terminate

    unsafe {
        POSTGRES_RESPONSE = Some(DbResponse {
            ptr: data.as_mut_ptr() as *mut c_void,
            length: data.len() * std::mem::size_of::<u16>(),
        });

        STRINGS.push_back(data);
    };
}

#[no_mangle]
pub extern "stdcall" fn get_data_len() -> i32 {
    unsafe {
        if let Some(data) = &POSTGRES_RESPONSE {
            data.length.try_into().expect("превышено максимальное значение i32 при конвертации беззнакового типа")
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "stdcall" fn get_data_ptr() -> *mut c_void {
    unsafe {
        if let Some(data) = &POSTGRES_RESPONSE {
            data.ptr
        } else {
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "stdcall" fn free_data(ptr: *mut c_void) {
    unsafe {
        if ptr.is_null() {
            return;
        }
        STRINGS.pop_front(); // assumes strings are freed in order they were got
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
