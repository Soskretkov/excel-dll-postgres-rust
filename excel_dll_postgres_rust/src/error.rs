use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt;
use std::string::FromUtf16Error;
use tokio_postgres::error::SqlState;

#[derive(Debug)]
pub enum Error {
    InvalidUtf16OnInput(FromUtf16Error),
    ServerNotAvailable,
    DbConnection(tokio_postgres::Error),
    SqlExecution(tokio_postgres::Error),
    RuntimeCreation(std::io::Error),
    DbTypeConversion {
        err: tokio_postgres::Error,
        column_type: tokio_postgres::types::Type,
    },
    DbTypeSupport(tokio_postgres::types::Type),
    Serialization(serde_json::Error),
    Deserialization(serde_json::Error),
    InternalLogic(String),
}

impl Error {
    // NNxx - Уникальный код ошибки.
    // xx1x - Внутренние ошибки: проблемы, слабо связанные с внешним миром.
    // xx2x - Внешние ошибки: ошибки, возникшие из-за некорректных данных на входе или действий пользователя.
    // xx3x - Ошибки при взаимодействии с базой данных.
    // xxx0 - Пользователя не нужно грузить деталями предоставив абстрактное описание.
    // xxx1 - Пользователю стоит показать общее описание.
    // xxx2 - Пользователю стоит показать общее описание и технические детали

    fn code(&self) -> &'static str {
        match self {
            Error::InvalidUtf16OnInput(_) => "0020",
            Error::ServerNotAvailable => "0131",
            Error::DbConnection(_) => "0132",
            Error::SqlExecution(_) => "0222",
            Error::DbTypeConversion { .. } => "0310",
            Error::DbTypeSupport(_) => "0431",
            Error::RuntimeCreation(_) => "0510",
            Error::Serialization(_) => "0610",
            Error::Deserialization(_) => "0720",
            Error::InternalLogic(_) => "0810",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidUtf16OnInput(_) => write!(f, "Не удалось конвертировать запрос в UTF-16"),
            Error::ServerNotAvailable => write!(f, "Сервер недоступен"),
            Error::DbConnection(_) => write!(f, "Внешняя база данных отвергает подключение"),
            Error::SqlExecution(_) => write!(f, "Не удалось выполнить SQL-запрос"),
            Error::DbTypeConversion { column_type, .. } => {
                write!(
                    f,
                    "Не удалось конвертировать тип базы данных '{}' в rust-тип",
                    column_type.name()
                )
            }
            Error::DbTypeSupport(column_type) => write!(
                f,
                "Тип столбца базы данных '{}' не поддерживается",
                column_type.name()
            ),
            Error::RuntimeCreation(_) => write!(f, "Не удалось создать рантайм Tokio"),
            Error::Serialization(_) => {
                write!(f, "Не удалось сериализовать ответ БД в JSON-формат")
            }
            Error::Deserialization(_) => write!(f, "Не валидные аргументы переданы в dll"),
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
        let tech_descr = match self {
            Error::InvalidUtf16OnInput(err) => Some(err.to_string()),
            Error::ServerNotAvailable => None,
            Error::DbConnection(err) => Some(err.to_string()),
            Error::SqlExecution(err) => Some(err.to_string()),
            Error::DbTypeConversion { err, .. } => Some(err.to_string()),
            Error::DbTypeSupport(_) => None,
            Error::RuntimeCreation(err) => Some(err.to_string()),
            Error::Serialization(err) => Some(err.to_string()),
            Error::Deserialization(err) => Some(err.to_string()),
            Error::InternalLogic(err) => Some(err.to_string()),
        };

        let mut s = serializer.serialize_struct("ExportError", 3)?;
        s.serialize_field("code", &self.code())?;
        s.serialize_field("descr", &self.to_string())?;
        match tech_descr {
            Some(descr) => s.serialize_field("tech_descr", &Some(&descr as &str))?,
            None => s.serialize_field("tech_descr", &None::<&str>)?,
        }

        s.end()
    }
}