use crate::db::Db;

pub struct Context<'a> {
    pub db: &'a Db,
}
