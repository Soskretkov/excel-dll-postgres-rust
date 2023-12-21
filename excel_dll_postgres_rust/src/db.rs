use super::api::ApiRequest;
use super::error::Error;
use serde::Deserialize;
use std::env;
use std::fs;
use tokio::runtime;
use tokio_postgres::{NoTls, Row};

pub fn get_database_response(
    excel_requests: &[ApiRequest],
    db_conect_params: ConectParams,
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

#[derive(Deserialize)]
pub struct ConectParams {
    pub host: String,
    pub db_name: String,
    pub user: String,
    pub password: String,
}

pub fn get_connection_params() -> ConectParams {
    // это только для теста, не храните реальные данные в dll!
    ConectParams {
        host: "localhost".to_string(),
        db_name: "el_dabaa".to_string(),
        user: "sdo_bot_readonly".to_string(),
        password: "Aa456456".to_string(),
    }
}

pub fn get_encrypt_connection_params() -> ConectParams {
    // Загрузка содержимого файла во время выполнения (предполагается что файл размещен где .dll)
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let encrypted_file_path = current_dir.join("encrypted.txt");
    let encrypted_file_content =
        fs::read(&encrypted_file_path).expect("Failed to read encrypted file");

    // Хардкодим ключ внутрь .dll
    let encryption_key = include_bytes!("../../encryption_key.txt");

    unimplemented!()
}
