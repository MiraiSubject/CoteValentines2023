use jammdb::Bucket;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Letter {
    pub anon: bool,
    pub recipient: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub user_id: String,
    pub letters: (Option<Letter>, Option<Letter>),
}

impl User {
    pub fn id(&self) -> String {
        self.user_id.clone()
    }
}

pub enum DbError {
    ItemNotFound,
    CantDeserialize,
    CantSerialize,
    ItemCantWrite,
}

impl User {
    pub fn get(bucket: &Bucket, user_id: &str) -> Result<Self, DbError> {
        match bucket.get(user_id) {
            None => Err(DbError::ItemNotFound),
            Some(data) => {
                Ok(rmp_serde::from_slice(data.kv().value())
                    .map_err(|_| DbError::CantDeserialize)?)
            }
        }
    }

    pub fn save(&self, bucket: &Bucket) -> Result<(), DbError> {
        bucket
            .put(
                self.id(),
                rmp_serde::to_vec(&self).map_err(|_| DbError::CantSerialize)?,
            )
            .map_err(|_| DbError::ItemCantWrite)
            .map(|_| ())
    }
}

pub struct NoSpaceError;
impl User {
    pub fn letter_added(&self, new: Letter) -> Result<Self, NoSpaceError> {
        match self.letters.clone() {
            (None, None) => Ok(User {
                user_id: self.id(),
                letters: (Some(new), None),
            }),
            (Some(previous), None) => Ok(User {
                user_id: self.id(),
                letters: (Some(previous), Some(new)),
            }),
            (None, Some(previous)) => Ok(User {
                user_id: self.id(),
                letters: (Some(new), Some(previous)),
            }),
            (Some(_), Some(_)) => Err(NoSpaceError),
        }
    }
}



#[cfg(test)]
mod test {
    #[test]
    fn populate_db() {
        use super::{Letter, User};
        use jammdb::DB;

        let user1 = User {
            user_id: "12345".to_owned(),
            letters: (None, None),
        };

        let user2 = User {
            user_id: "54321".to_owned(),
            letters: (
                Some(Letter {
                    anon: false,
                    recipient: "12345".to_owned(),
                    content: "143 < 3".to_owned(),
                }),
                None,
            ),
        };

        if let Ok(db) = DB::open("test-database.db") {
            let tx = db.tx(true).unwrap();

            // create a bucket to store a map of first names to last names
            let names_bucket = tx.create_bucket("users").unwrap();
            names_bucket
                .put(user1.user_id.clone(), serde_json::to_vec(&user1).unwrap())
                .unwrap();
            names_bucket
                .put(user2.user_id.clone(), serde_json::to_vec(&user2).unwrap())
                .unwrap();

            // commit the changes so they are saved to disk
            tx.commit().unwrap();
        };
        if let Ok(db) = DB::open("test-database.db") {
            let tx = db.tx(false).unwrap();
            // get the bucket we created in the last transaction
            let names_bucket = tx.get_bucket("users").unwrap();
            // get the key/ value pair we inserted into the bucket
            if let Some(data) = names_bucket.get(user1.user_id.clone()) {
                assert!(data.is_kv());
                assert_eq!(data.kv().value(), b"Jarrus");
            }
        }
    }
}
