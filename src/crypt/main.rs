// use std::env;
use std::fs::File;
use std::io::Write;

use super::encrypt;

fn main() {
    // Получить аргументы командной строки. 
    // Первый аргумент - это путь к файлу конфигурации, 
    // Второй - путь к выходному файлу
    // Третий - ключ для шифрования
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let output_path = &args[2];
    let encryption_key = &args[3];

    // Прочитать файл конфигурации и шифровать его содержимое
    let config_data = std::fs::read_to_string(config_path).expect("Could not read config file");
    let encrypted_data = encrypt(&config_data, encryption_key);

    // Записать зашифрованные данные в файл
    let mut file = File::create(output_path).expect("Could not create output file");
    file.write_all(&encrypted_data).expect("Could not write to output file");
}
