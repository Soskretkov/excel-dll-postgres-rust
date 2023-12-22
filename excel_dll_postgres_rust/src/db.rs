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
    excel_requests: &[ApiRequest],
    db_conect_params: Login,
) -> Result<Vec<Result<Vec<Row>, Error>>, Error> {
    // строка параметров для соединения с БД
    let parameter_string = format!(
        "host={} dbname={} user={} password={}",
        db_conect_params.host,
        db_conect_params.db_name,
        db_conect_params.user,
        db_conect_params.password
    );

    // Tokio автоматически создает рантайм для асинхронных операций, но ниже это делается вручную - код не в асинхронной среде
    let rt = runtime::Runtime::new().map_err(Error::TokioRuntimeCreation)?;

    // NoTls - не требуетя защищенного соединения, что приемлемо в защищенной среде
    let (client, connection) = rt
        .block_on(tokio_postgres::connect(&parameter_string, NoTls))
        .map_err(Error::DBConnection)?;

    // запускает асинхронную задачу, которая ожидает завершения соединения с БД. Если ошибка, она будет записана в стандартный поток ошибок
    rt.spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut res: Vec<Result<Vec<Row>, Error>> = Vec::new();
    for request in excel_requests {
        res.push(
            rt.block_on(client.query(&request.sql_query, &[]))
                .map_err(Error::SqlExecution),
        );
    }

    Ok(res)
}

pub fn get_db_auth_data() -> Login {
    let params_file_content = include_str!("../../unencrypted.txt");
    let params:Login = serde_json::from_str(params_file_content).unwrap();

    params
}

fn get_encrypt_db_auth_data() -> Login {
    // Загрузка содержимого файла во время выполнения (предполагается что файл размещен где .dll)
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let encrypted_file_path = current_dir.join("encrypted.txt");
    let encrypted_file_content =
        fs::read(&encrypted_file_path).expect("Failed to read encrypted file");

    // Хардкодим ключ внутрь .dll. Файл ожидается в корне проекта
    // let encryption_key = include_bytes!("../../encryption_key.txt");

    unimplemented!()
}
