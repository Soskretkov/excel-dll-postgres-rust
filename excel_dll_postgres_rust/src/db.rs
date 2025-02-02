use super::api::ApiRequest;
use super::error::Error;
use serde::Deserialize;
use std::env;
use std::fs;
use tokio::runtime;
use tokio_postgres::{NoTls, Row};

#[derive(Deserialize)]
pub struct Login {
    pub host: String,
    #[serde(rename = "dbName")]
    pub db_name: String,
    pub user: String,
    pub password: String,
}

pub fn get_database_response(
    requests: &[ApiRequest],
    db_conect_params: Login,
) -> Result<Vec<Result<Vec<Row>, Error>>, Error> {
    // строка параметров для соединения с БД
    let mut parameter_string = format!(
        "host={} dbname={} user={}",
        db_conect_params.host, db_conect_params.db_name, db_conect_params.user,
    );

    if !db_conect_params.password.is_empty() {
        parameter_string = format!("{parameter_string} password={}", db_conect_params.password);
    }

    // Tokio автоматически создает рантайм для асинхронных операций, но ниже это делается вручную - код не в асинхронной среде
    let rt = runtime::Runtime::new().map_err(Error::RuntimeCreation)?;

    // Подключаемся к БД
    // NoTls - не требуетя защищенного соединения, что приемлемо в защищенной среде
    let (client, connection) = rt
        .block_on(tokio_postgres::connect(&parameter_string, NoTls))
        .map_err(|e| match e.as_db_error() {
            None => Error::ServerNotAvailable,
            // рукав Some может не браться, если в файле postgresql.conf LC_MESSAGES не "English_United States.1252"
            Some(_) => Error::DbConnection(e),
        })?;

    let mut res: Vec<Result<Vec<Row>, Error>> = Vec::new();
    for request in requests {
        res.push(
            rt.block_on(client.query(&request.sql_query, &[]))
                .map_err(Error::SqlExecution),
        );
    }

    // Явно ждём завершения соединения перед выходом из функции
    rt.block_on(connection).map_err(Error::DbConnection)?;

    Ok(res)
}

pub fn get_db_auth_data() -> Login {
    // Загрузка параметров подключения к БД из файла во время компиляции. Содержимое файла, образец:
    // {
    //   "host": "localhost",
    //   "dbName": "el_dabaa",
    //   "user": "postgres",
    //   "password": ""
    // }
    let params_file_content = include_str!("../../unencrypted/unencrypted.txt");
    let params: Login = serde_json::from_str(params_file_content).unwrap();

    params
}

fn _get_encrypt_db_auth_data() -> Login {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let encrypted_file_path = current_dir.join("encrypted.txt");
    let _encrypted_file_content =
        fs::read(&encrypted_file_path).expect("Failed to read encrypted file");

    // Хардкодим ключ внутрь .dll. Файл ожидается в корне проекта
    // let encryption_key = include_bytes!("../../encryption_key.txt");

    unimplemented!()
}
