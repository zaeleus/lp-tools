use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::{Medium, NewMedium, ReleaseId};

pub struct MediumRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> MediumRepository<'a> {
    pub fn new(connection: &PgConnection) -> MediumRepository<'_> {
        MediumRepository { connection }
    }

    pub fn create(
        &self,
        release_id: ReleaseId,
        kind: i32,
        position: i16,
        name: Option<&str>,
    ) -> Medium {
        use crate::schema::media;

        let now = Utc::now().naive_utc();

        let new_medium = NewMedium {
            release_id,
            kind,
            position,
            name,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(media::table)
            .values(&new_medium)
            .get_result(self.connection)
            .expect("Error creating new medium")
    }
}
