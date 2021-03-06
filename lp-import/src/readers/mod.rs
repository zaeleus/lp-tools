pub mod album;
pub mod artist;
pub mod artist_credit;
pub mod artist_credit_name;
pub mod contribution;
pub mod medium;
pub mod membership;
pub mod release;
pub mod song;
pub mod track;
pub mod tracklist;

#[derive(Debug)]
pub enum Error {
    Map(String),
    Parse(String),
}
