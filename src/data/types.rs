use rusqlite::types::{FromSql, ToSql};
use anyhow::Result;

pub trait Entity: Send + Sync {
    type Id: FromSql + ToSql + Clone;
    fn get_id(&self) -> &Self::Id;
}

pub trait Repository<T: Entity>: Send + Sync {
    fn get_by_id(&self, id: &T::Id) -> Result<Option<T>>;
    fn save(&self, entity: &T) -> Result<()>;
    fn update(&self, entity: &T) -> Result<()>;
    fn delete(&self, id: &T::Id) -> Result<()>;
}