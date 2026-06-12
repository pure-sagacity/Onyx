// @generated automatically by Diesel CLI.

diesel::table! {
    projects (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    secrets (id) {
        id -> Nullable<Integer>,
        project_id -> Text,
        name -> Text,
        environment -> Text,
        value -> Text,
        nonce -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(secrets -> projects (project_id));

diesel::allow_tables_to_appear_in_same_query!(projects, secrets,);
