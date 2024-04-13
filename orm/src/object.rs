use crate::{
    data::DataType,
    storage::{Row, RowSlice},
};

use std::any::Any;

////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any {
    const SCHEMA: Schema;
    fn to_row(&self) -> Row;
    fn from_row(row: &RowSlice) -> Self;
    fn schema() -> Schema;
}

////////////////////////////////////////////////////////////////////////////////

pub trait Store: Any {
    fn to_row(&self) -> Row;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_schema(&self) -> Schema;
}

impl<T: Object> Store for T {
    fn to_row(&self) -> Row {
        self.to_row()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_schema(&self) -> Schema {
        T::schema()
    }
}

pub struct Schema {
    pub object_name: &'static str,
    pub table_name: &'static str,
    pub fields: &'static [Field],
}

pub struct Field {
    pub feild_name: &'static str,
    pub column_name: &'static str,
    pub feild_type: DataType,
}

impl Schema {
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|&f| f.column_name == name)
    }
}
