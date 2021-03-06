use chrono::{NaiveDate, NaiveDateTime};

use crate::models::AlbumId;
use crate::schema::releases;

pub type ReleaseId = i32;

#[derive(Debug, Queryable)]
pub struct Release {
    pub id: AlbumId,
    pub album_id: AlbumId,
    pub released_on: NaiveDate,
    pub country: Option<String>,
    pub catalog_number: Option<String>,
    pub disambiguation: Option<String>,
    pub artwork_data: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "releases"]
pub struct NewRelease<'a> {
    pub album_id: AlbumId,
    pub released_on: NaiveDate,
    pub country: Option<&'a str>,
    pub catalog_number: Option<&'a str>,
    pub disambiguation: Option<&'a str>,
    pub artwork_data: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
