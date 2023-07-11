
use std::slice;
// структура с данными по которым vba получит строку из памяти, #[repr(C)] зафиксирует поля в порядке как задал программист

#[repr(C)]
pub struct StringWrapForVBA {
    ptr: *mut u16,
    length_in_bytes: i32,
    _data: Box<Vec<u16>>, // это поле не будет читаться VBA и оно тут для выравнивания времени жизни с полем "ptr" для того чтобы vba читал действительный "ptr"
}

pub fn get_string_from_vba(bstr_ptr: *const u16) -> String {
    // вычисление длины строки
    let bstr_len_ptr = unsafe { bstr_ptr.offset(-2) as *const u32 };

    let bstr_len_in_bytes: u32 = unsafe { std::ptr::read_unaligned(bstr_len_ptr) };

    // создание среза: второй параметр это число итераций, длина итерантов определяется по типу указателя (2 байта у нас)
    let slice = unsafe { slice::from_raw_parts(bstr_ptr, (bstr_len_in_bytes / 2) as usize) }; // создание среза
    String::from_utf16(slice).unwrap()
}

pub fn get_string_ptr_for_vba(text: String) -> *mut StringWrapForVBA {
    //.encode_utf16() правильно обрабатывает суррогатные пары Unicode (возвращает итератор 16-битных юнитов кодировки UTF-16).
    let data: Vec<u16> = text.encode_utf16().collect();
    let length_in_bytes: i32 = (text.encode_utf16().count() * std::mem::size_of::<u16>())
        .try_into()
        .unwrap();

    let boxed_data = Box::new(data);
    let ptr = boxed_data.as_ptr() as *mut u16;

    let sending_data = StringWrapForVBA {
        ptr,
        length_in_bytes,
        _data: boxed_data,
    };
    Box::into_raw(Box::new(sending_data))
}
