use crate::{
    data::{DataType, Value},
    error::{Error, MapError, Result},
    object::Schema,
    ObjectId,
};

use rusqlite::params_from_iter;

use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////

pub type Row<'a> = Vec<Value<'a>>;
pub type RowSlice<'a> = [Value<'a>];

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait StorageTransaction {
    fn table_exists(&self, table: &str) -> Result<bool>;
    fn create_table(&self, schema: &Schema) -> Result<()>;

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId>;
    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()>;
    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>>;
    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

impl<'a> StorageTransaction for rusqlite::Transaction<'a> {
    fn table_exists(&self, table: &str) -> Result<bool> {
        let mut stmt = self.prepare("SELECT 1 FROM sqlite_master WHERE name = :name")?;
        let result = stmt.exists(&[(":name", table)]);
        result.map_err(Error::from)
    }

    fn create_table(&self, schema: &Schema) -> Result<()> {
        let e = "".to_string();
        let sql = format!(
            "CREATE TABLE {0} (id INTEGER PRIMARY KEY AUTOINCREMENT{1})",
            schema.table_name,
            schema
                .fields
                .iter()
                .fold(e, |acc, f2| format!(
                    "{0}, {1} {2}",
                    acc,
                    f2.column_name,
                    f2.feild_type.to_sql()
                ))
                .as_str()
        );
        self.execute(&sql, []).map_table_error(schema)?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        let sql = match schema.fields.len() {
            0 => format!("INSERT INTO {0} DEFAULT VALUES", schema.table_name),
            _ => format!(
                "INSERT INTO {0}({1}) VALUES ({2})",
                schema.table_name,
                schema
                    .fields
                    .iter()
                    .map(|f| f.column_name)
                    .collect::<Vec<&str>>()
                    .join(", "),
                vec!["?"; schema.fields.len()].join(", ")
            ),
        };
        self.execute(&sql, params_from_iter(row.iter()))
            .map_table_error(schema)?;
        let id = self.last_insert_rowid();
        Ok(ObjectId(id))
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        let sql = format!(
            "UPDATE {0} SET {1} WHERE id = {2}",
            schema.table_name,
            schema
                .fields
                .iter()
                .map(|f| format!("{0} = ?", f.column_name))
                .collect::<Vec<String>>()
                .join(", "),
            id.0
        );
        self.execute(&sql, params_from_iter(row.iter()))
            .map_row_error(schema, id)?;
        Ok(())
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        let sql = match schema.fields.len() {
            0 => format!("SELECT * FROM {0} WHERE id = {1}", schema.table_name, id.0),
            _ => format!(
                "SELECT {0} FROM {1} WHERE id = {2}",
                schema
                    .fields
                    .iter()
                    .map(|f| f.column_name)
                    .collect::<Vec<&str>>()
                    .join(", "),
                schema.table_name,
                id.0
            ),
        };
        let x = self
            .query_row_and_then(&sql, [], |row| {
                schema
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| match field.feild_type {
                        DataType::String => {
                            Ok(Value::String(Cow::from(row.get::<usize, String>(i)?)))
                        }
                        DataType::Bytes => {
                            Ok(Value::Bytes(Cow::from(row.get::<usize, Vec<u8>>(i)?)))
                        }
                        DataType::Int64 => Ok(Value::Int64(row.get(i)?)),
                        DataType::Float64 => Ok(Value::Float64(row.get(i)?)),
                        DataType::Bool => Ok(Value::Bool(row.get(i)?)),
                    })
                    .collect()
            })
            .map_row_error(schema, id)?;
        Ok(x)
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        let sql = format!("DELETE FROM {0} WHERE id == {1}", schema.table_name, id.0);
        self.execute(&sql, []).map_row_error(schema, id)?;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        self.execute("COMMIT", [])?;
        Ok(())
    }

    fn rollback(&self) -> Result<()> {
        self.execute("ROLLBACK", [])?;
        Ok(())
    }
}
