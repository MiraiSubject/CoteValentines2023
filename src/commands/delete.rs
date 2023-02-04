use std::fmt::format;

use diesel::prelude::*;

fn delete_letter(conn: &mut SqliteConnection) {
    use crate::schema::letters::dsl::*;

    let res = diesel::delete(letters.filter(id.eq(1)))
        .execute(conn)
        .map_err(|e| {
            format!("Error {e}");
        });
}
