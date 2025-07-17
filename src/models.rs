use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::runners_saturation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RunnersSaturationScheme {
    pub id: i32,
    pub rid: i64,
    pub name: String,
    pub busy: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::runners_saturation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRunnersSaturation {
    pub rid: i64,
    pub name: String,
    pub busy: bool,
    pub created_at: NaiveDateTime,
}
