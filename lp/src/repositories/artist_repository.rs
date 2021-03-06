use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::ArtistName;
use crate::models::{Artist, ArtistId, NewArtist};
use crate::PartialDate;

pub struct ArtistRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> ArtistRepository<'a> {
    pub fn new(connection: &PgConnection) -> ArtistRepository<'_> {
        ArtistRepository { connection }
    }

    pub fn find(&self, id: ArtistId) -> Option<Artist> {
        use crate::schema::artists::dsl::artists;
        artists.find(id).first(self.connection).ok()
    }

    pub fn search(&self, query: &str) -> Vec<Artist> {
        use crate::schema::{artist_names, artists};

        let pattern = format!("%{}%", query);

        artists::table
            .inner_join(artist_names::table)
            .filter(artist_names::name.like(&pattern))
            .load::<(Artist, ArtistName)>(self.connection)
            .unwrap()
            .into_iter()
            .map(|(a, _)| a)
            .collect()
    }

    pub fn create(
        &self,
        kind: i32,
        country: &str,
        started_on: Option<PartialDate>,
        ended_on: Option<PartialDate>,
        disambiguation: Option<&str>,
    ) -> Artist {
        use crate::schema::artists;

        let started_on = started_on.unwrap_or_default();
        let ended_on = ended_on.unwrap_or_default();

        let now = Utc::now().naive_utc();

        let new_artist = NewArtist {
            kind,
            country,
            disambiguation,
            started_on_year: started_on.year,
            started_on_month: started_on.month,
            started_on_day: started_on.day,
            ended_on_year: ended_on.year,
            ended_on_month: ended_on.month,
            ended_on_day: ended_on.day,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(artists::table)
            .values(&new_artist)
            .get_result(self.connection)
            .expect("Error creating new artist")
    }
}
