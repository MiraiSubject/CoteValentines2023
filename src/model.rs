use crate::schema::*;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Letter {
    pub id: i32,
    pub recipient: String,
    pub sender: String,
    pub anon: bool,
    pub content: String,
    pub message_id: String,
    pub sender_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = letters)]
pub struct NewLetter<'a> {
    pub recipient: &'a str,
    pub sender: &'a str,
    pub anon: bool,
    pub content: &'a str,
    pub message_id: &'a str,
    pub sender_id: &'a str,
}

#[derive(Queryable, Insertable)]
pub struct Recipient {
    pub fullname: String,
    pub is_real: bool,
}
