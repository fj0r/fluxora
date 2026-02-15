use super::db::Model;
use axum::extract::FromRef;

pub type Db = Model;

#[derive(Debug, Clone)]
pub struct Shared {
    pub db: Db,
}

impl FromRef<Shared> for Db {
    fn from_ref(input: &Shared) -> Self {
        input.db.clone()
    }
}

impl Shared {
    pub fn new(db: Model) -> Self {
        Self { db }
    }
}
