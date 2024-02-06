use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt;
use std::string::FromUtf16Error;

#[derive(Debug)]
pub enum Error {
    InvalidUtf16OnInput(FromUtf16Error),
    DBConnection(tokio_postgres::Error),
    SqlExecution(tokio_postgres::Error),
    TokioRuntimeCreation(std::io::Error),
    DataRetrieval(tokio_postgres::Error),
    JsonSerialization(serde_json::Error),
    JsonDeserialization(serde_json::Error),
    InternalLogic(String),
}

impl Error {
    // 1xxx - Внутренние ошибки: проблемы, слабо связанные с внешним миром.
    // 2xxx - Внешние ошибки: ошибки, возникшие из-за некорректных данных на входе или действий пользователя.
    // 3xxx - Ошибки состояния базы данных: проблемы при взаимодействии с базой данных.
    // x0xx - Пользователя не нужно грузить деталями предоставив абстрактное описание.
    // x1xx - Пользователю стоит показать общее описание.
    // x2xx - Пользователю стоит показать общее описание и технические детали
    // xxNN - Уникальный код ошибки.

    fn code(&self) -> u32 {
        match self {
            Error::InvalidUtf16OnInput(_) => 2000,
            Error::DBConnection(_) => 3101,
            Error::SqlExecution(_) => 2202,
            Error::DataRetrieval(_) => 1003,
            Error::TokioRuntimeCreation(_) => 1005,
            Error::JsonSerialization(_) => 1006,
            Error::JsonDeserialization(_) => 2007,
            Error::InternalLogic(_) => 1008,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidUtf16OnInput(_) => write!(f, "Не удалось конвертировать запрос в UTF-16"),
            Error::DBConnection(_) => write!(f, "База данных недоступна"),
            Error::SqlExecution(_) => write!(f, "Не удалось выполнить SQL-запрос"),
            Error::DataRetrieval(_) => {
                write!(f, "Не удалось конвертировать тип базы данных в rust-тип")
            }
            Error::TokioRuntimeCreation(_) => write!(f, "Не удалось создать рантайм Tokio"),
            Error::JsonSerialization(_) => write!(f, "Не удалось сериализовать ответ БД в JSON-формат"),
            Error::JsonDeserialization(_) => write!(f, "Не валидные аргументы переданы в dll"),
            Error::InternalLogic(_) => write!(f, "Логическая ошибка в dll"),
        }
    }
}

impl From<FromUtf16Error> for Error {
    fn from(error: FromUtf16Error) -> Self {
        Error::InvalidUtf16OnInput(error)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ExportError", 3)?;
        s.serialize_field("code", &self.code())?;
        s.serialize_field("descr", &self.to_string())?;
        s.serialize_field("tech_descr", {
            &match self {
                Error::InvalidUtf16OnInput(err) => err.to_string(),
                Error::DBConnection(err) => err.to_string(),
                Error::SqlExecution(err) => err.to_string(),
                Error::DataRetrieval(err) => err.to_string(),
                Error::TokioRuntimeCreation(err) => err.to_string(),
                Error::JsonSerialization(err) => err.to_string(),
                Error::JsonDeserialization(err) => err.to_string(),
                Error::InternalLogic(err) => err.to_string(),
            }
        })?;

        s.end()
    }
}
