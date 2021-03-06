use chrono::Utc;
use diesel::prelude::*;
use diesel::{self, PgConnection};

use crate::models::{ArtistCreditId, ArtistId, Membership, NewMembership};
use crate::PartialDate;

pub struct MembershipRepository<'a> {
    connection: &'a PgConnection,
}

impl<'a> MembershipRepository<'a> {
    pub fn new(connection: &PgConnection) -> MembershipRepository<'_> {
        MembershipRepository { connection }
    }

    pub fn create(
        &self,
        artist_id: ArtistId,
        artist_credit_id: ArtistCreditId,
        started_on: Option<PartialDate>,
        ended_on: Option<PartialDate>,
    ) -> Membership {
        use crate::schema::memberships;

        let started_on = started_on.unwrap_or_default();
        let ended_on = ended_on.unwrap_or_default();

        let now = Utc::now().naive_utc();

        let new_membership = NewMembership {
            group_id: artist_id,
            artist_credit_id,
            started_on_year: started_on.year,
            started_on_month: started_on.month,
            started_on_day: started_on.day,
            ended_on_year: ended_on.year,
            ended_on_month: ended_on.month,
            ended_on_day: ended_on.day,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(memberships::table)
            .values(&new_membership)
            .get_result(self.connection)
            .expect("Error creating new membership")
    }
}
