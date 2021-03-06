use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::{ArtistCreditId, NewSong, Song};

pub struct SongRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> SongRepository<'a> {
    pub fn new(connection: &PgConnection) -> SongRepository<'_> {
        SongRepository { connection }
    }

    pub fn create(&self, artist_credit_id: ArtistCreditId) -> Song {
        use crate::schema::songs;

        let now = Utc::now().naive_utc();

        let new_song = NewSong {
            artist_credit_id,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(songs::table)
            .values(&new_song)
            .get_result(self.connection)
            .expect("Error creating new song")
    }
}
