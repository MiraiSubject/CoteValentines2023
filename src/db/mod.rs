pub mod model;

pub trait Database<T> {
    type DbError;

    fn get(&self, user_id: &str) -> Result<T, Self::DbError>;
    fn save(&self, user: &T) -> Result<(), Self::DbError>;
}

