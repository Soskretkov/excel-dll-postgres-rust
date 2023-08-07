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
    JsonSerialization(serde_json::Error),
    JsonDeserialization(serde_json::Error),
}

impl Error {
    // 1xxx - Внутренние ошибки: проблемы, слабо связанные с внешним миром.
    // 2xxx - Внешние ошибки: ошибки, возникшие из-за некорректных данных на входе или действий пользователя.
    // 3xxx - Ошибки состояния базы данных: проблемы при взаимодействии с базой данных.
    // x0xx - Пользователя не нужно грузить деталями.
    // x1xx - Пользователя стоит погрузить в детали.
    // xxNN - Уникальный код ошибки.

    fn code(&self) -> u32 {
        match self {
            Error::InvalidUtf16OnInput(_) => 2000,
            Error::DBConnection(_) => 3101,
            Error::SqlExecution(_) => 2102,
            Error::TokioRuntimeCreation(_) => 1003,
            Error::JsonSerialization(_) => 1004,
            Error::JsonDeserialization(_) => 2005,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidUtf16OnInput(_) => write!(f, "не удалось конвертировать запрос в UTF-16"),
            Error::DBConnection(_) => write!(f, "не удалось подключение к базе данных"),
            Error::SqlExecution(_) => write!(f, "не удалось выполнить SQL-запрос"),
            Error::TokioRuntimeCreation(_) => write!(f, "не удалось создать рантайм Tokio"),
            Error::JsonSerialization(_) => write!(f, "не удалось сериализовать ответ БД в JSON"),
            Error::JsonDeserialization(_) => write!(f, "не валидные аргументы переданы в dll"),
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
                Error::TokioRuntimeCreation(err) => err.to_string(),
                Error::JsonSerialization(err) => err.to_string(),
                Error::JsonDeserialization(err) => err.to_string(),
            }
        })?;

        s.end()
    }
}
