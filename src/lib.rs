// use std::ffi::CString;

// #[no_mangle]
// pub extern "stdcall" fn get_data() -> *const i8 {
//     let response_text = executeSqlQuery();
//     let data = CString::new(response_text).unwrap();
//     data.into_raw()
// }

// #[no_mangle]
// pub extern "stdcall" fn free_data(ptr: *mut i8) {
//     unsafe {
//         if ptr.is_null() {
//             return;
//         }
//         CString::from_raw(ptr)
//     };
// }

// fn executeSqlQuery() -> String {
//     let database_response = "SELECT * FROM ref_currency_type;".to_string();
//     database_response
// }










use std::os::raw::c_void;
use std::ffi::CString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::collections::VecDeque;

static mut STRINGS: VecDeque<Vec<u16>> = VecDeque::new();

#[no_mangle]
pub extern "stdcall" fn get_data() -> *mut c_void {
    let response_text = executeSqlQuery();
    let mut data: Vec<u16> = response_text.encode_utf16().collect();
    data.push(0); // null terminate
    let ptr = data.as_mut_ptr() as *mut c_void;
    unsafe { STRINGS.push_back(data) };
    ptr
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

fn executeSqlQuery() -> String {
    let database_response = "SELECT * FROM ref_currency_type;".to_string();
    database_response
}
