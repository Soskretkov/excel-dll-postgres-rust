use std::slice;
use std::string::FromUtf16Error;

// структура по которой vba получает строку, #[repr(C)] фиксирует поля в порядке как задал программист
#[repr(C)]
pub struct StringForVba {
    ptr: *mut u16,
    is_valid: bool,
    length_in_bytes: i32,
    _data: Vec<u16>, // это поле не будет читаться VBA и оно тут для выравнивания времени жизни с полем "ptr" для того чтобы vba читал действительный "ptr"

}

impl StringForVba {
    pub fn from_string(text: String) -> Self {
        //.encode_utf16() правильно обрабатывает суррогатные пары Unicode (возвращает итератор 16-битных юнитов кодировки UTF-16).
        let data: Vec<u16> = text.encode_utf16().collect();
        let length_in_bytes: i32 = (text.encode_utf16().count() * std::mem::size_of::<u16>())
            .try_into()
            .unwrap();

        let ptr = data.as_ptr() as *mut u16;

        StringForVba {
            ptr,
            is_valid: true,
            length_in_bytes,
            _data: data,
        }
    }

    pub fn validity_update(&mut self, is_valid: bool) {
        self.is_valid = is_valid;
    }

    pub fn into_raw(self) -> *mut StringForVba {
        Box::into_raw(Box::new(self))
    }
}

pub fn get_string_from_vba(bstr_ptr: *const u16) -> Result<String, FromUtf16Error> {
    // вычисление длины строки
    let bstr_len_ptr = unsafe { bstr_ptr.offset(-2) as *const u32 };

    let bstr_len_in_bytes: u32 = unsafe { std::ptr::read_unaligned(bstr_len_ptr) };

    // создание среза: второй параметр это число итераций, длина итерантов определяется по типу указателя (2 байта у нас)
    let slice = unsafe { slice::from_raw_parts(bstr_ptr, (bstr_len_in_bytes / 2) as usize) }; // создание среза
    String::from_utf16(slice)
}
