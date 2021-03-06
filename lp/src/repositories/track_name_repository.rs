use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::{NewTrackName, TrackId, TrackName};

pub struct TrackNameRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> TrackNameRepository<'a> {
    pub fn new(connection: &PgConnection) -> TrackNameRepository<'_> {
        TrackNameRepository { connection }
    }

    pub fn find_by_track_id(&self, id: TrackId) -> Vec<TrackName> {
        use crate::schema::track_names::dsl::{track_id, track_names};

        track_names
            .filter(track_id.eq(id))
            .load(self.connection)
            .expect("failed to load names")
    }

    pub fn create(
        &self,
        track_id: TrackId,
        name: &str,
        locale: &str,
        is_default: bool,
        is_original: bool,
    ) -> TrackName {
        use crate::schema::track_names;

        let now = Utc::now().naive_utc();

        let new_track_name = NewTrackName {
            track_id,
            name,
            locale,
            is_default,
            is_original,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(track_names::table)
            .values(&new_track_name)
            .get_result(self.connection)
            .expect("Error creating new track")
    }
}
