use crate::{data::DataType, object::Schema, ObjectId};

use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(Box<NotFoundError>),
    #[error(transparent)]
    UnexpectedType(Box<UnexpectedTypeError>),
    #[error(transparent)]
    MissingColumn(Box<MissingColumnError>),
    #[error("database is locked")]
    LockConflict,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error>),
}

fn check_missing(s: &str) -> Option<&'_ str> {
    if let Some(x) = s.strip_prefix("no such column: ") {
        return Some(x);
    }

    if let Some(x) = s.split("has no column named ").last() {
        return Some(x);
    }
    None
}

pub trait MapError<T> {
    fn map_row_error(self, schema: &Schema, id: ObjectId) -> std::result::Result<T, Error>;
    fn map_table_error(self, schema: &Schema) -> std::result::Result<T, Error>;
}

pub trait MatchError {
    fn match_error(self, schema: &Schema) -> Error;
    fn match_error_id(self, schema: &Schema, id: ObjectId) -> Error;
}

impl MatchError for rusqlite::Error {
    fn match_error_id(self, schema: &Schema, id: ObjectId) -> Error {
        match self {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(Box::new(NotFoundError {
                object_id: id,
                type_name: schema.object_name,
            })),
            _ => self.match_error(schema),
        }
    }
    fn match_error(self, schema: &Schema) -> Error {
        match self {
            rusqlite::Error::InvalidColumnType(_, column, t) => {
                let field = schema.get_field(&column).unwrap();
                Error::UnexpectedType(Box::new(UnexpectedTypeError {
                    type_name: schema.object_name,
                    attr_name: field.feild_name,
                    table_name: schema.table_name,
                    column_name: field.column_name,
                    expected_type: field.feild_type,
                    got_type: t.to_string(),
                }))
            }
            rusqlite::Error::SqliteFailure(e, op) => {
                if let rusqlite::ErrorCode::DatabaseBusy = e.code {
                    return Error::LockConflict;
                }
                let s = op.unwrap();
                if let Some(a) = check_missing(s.as_str()) {
                    if let Some(f) = schema.get_field(a) {
                        return Error::MissingColumn(Box::new(MissingColumnError {
                            type_name: schema.object_name,
                            attr_name: f.feild_name,
                            table_name: schema.table_name,
                            column_name: f.column_name,
                        }));
                    }
                }
                Error::Storage(Box::new(e))
            }
            _ => Error::Storage(Box::new(self)),
        }
    }
}

impl<T> MapError<T> for rusqlite::Result<T> {
    fn map_row_error(self, schema: &Schema, id: ObjectId) -> std::result::Result<T, Error> {
        self.map_err(|err| err.match_error_id(schema, id))
    }
    fn map_table_error(self, schema: &Schema) -> std::result::Result<T, Error> {
        self.map_err(|err| err.match_error(schema))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(_, _) => Error::LockConflict,
            _ => Error::Storage(Box::new(err)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("object is not found: type '{type_name}', id {object_id}")]
pub struct NotFoundError {
    pub object_id: ObjectId,
    pub type_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "invalid type for {type_name}::{attr_name}: expected equivalent of {expected_type:?}, \
    got {got_type} (table: {table_name}, column: {column_name})"
)]
pub struct UnexpectedTypeError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub expected_type: DataType,
    pub got_type: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "missing a column for {type_name}::{attr_name} \
    (table: {table_name}, column: {column_name})"
)]
pub struct MissingColumnError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;
