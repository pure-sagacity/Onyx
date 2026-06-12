use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::secrets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Secret {
    pub id: Option<i32>,
    pub project_id: String,
    pub name: String,
    pub environment: String,
    pub value: String,
    pub nonce: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}
