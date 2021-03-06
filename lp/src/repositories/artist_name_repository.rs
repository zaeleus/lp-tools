use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::{ArtistId, ArtistName, NewArtistName};

pub struct ArtistNameRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> ArtistNameRepository<'a> {
    pub fn new(connection: &PgConnection) -> ArtistNameRepository<'_> {
        ArtistNameRepository { connection }
    }

    pub fn find_by_artist_id(&self, id: ArtistId) -> Vec<ArtistName> {
        use crate::schema::artist_names::dsl::{artist_id, artist_names};

        artist_names
            .filter(artist_id.eq(id))
            .load(self.connection)
            .expect("failed to load names")
    }

    pub fn create(
        &self,
        artist_id: ArtistId,
        name: &str,
        locale: &str,
        is_default: bool,
        is_original: bool,
    ) -> ArtistName {
        use crate::schema::artist_names;

        let now = Utc::now().naive_utc();

        let new_artist_name = NewArtistName {
            artist_id,
            name,
            locale,
            is_default,
            is_original,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(artist_names::table)
            .values(&new_artist_name)
            .get_result(self.connection)
            .expect("Error creating new artist")
    }
}
