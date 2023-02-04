use diesel::prelude::*;
use crate::schema::letters as table;

#[derive(Queryable)]
pub struct Letter {
    pub id: i32,
    pub recipient: String,
    pub sender: String,
    pub anon: bool,
    pub content: String,
}

#[derive(Insertable)]
#[diesel(table_name = table)]
pub struct NewLetter<'a> {
    pub recipient: &'a str,
    pub sender: &'a str,
    pub anon: bool,
    pub content: &'a str,

}