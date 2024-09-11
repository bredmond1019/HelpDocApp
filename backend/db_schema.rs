// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vector"))]
    pub struct Vector;
}

diesel::table! {
    articles (id) {
        id -> Uuid,
        collection_id -> Uuid,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        slug -> Varchar,
        html_content -> Nullable<Text>,
        markdown_content -> Nullable<Text>,
        version -> Int4,
        #[max_length = 255]
        last_edited_by -> Nullable<Varchar>,
        #[max_length = 255]
        helpscout_collection_id -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        helpscout_article_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    collections (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 255]
        slug -> Varchar,
        #[max_length = 255]
        helpscout_collection_id -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    content_versions (id) {
        id -> Uuid,
        article_id -> Nullable<Uuid>,
        version_number -> Int4,
        markdown_content -> Nullable<Text>,
        #[max_length = 255]
        edited_by -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Vector;

    embeddings (id) {
        id -> Uuid,
        article_id -> Uuid,
        embedding_vector -> Vector,
    }
}

diesel::joinable!(articles -> collections (collection_id));
diesel::joinable!(content_versions -> articles (article_id));
diesel::joinable!(embeddings -> articles (article_id));

diesel::allow_tables_to_appear_in_same_query!(
    articles,
    collections,
    content_versions,
    embeddings,
);
