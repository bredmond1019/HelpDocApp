// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

    article_chunks (id) {
        id -> Uuid,
        article_id -> Uuid,
        content -> Text,
        is_title -> Bool,
        embedding_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

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
        paragraph_description -> Nullable<Text>,
        bullet_points -> Nullable<Array<Nullable<Text>>>,
        keywords -> Nullable<Array<Nullable<Text>>>,
        paragraph_description_embedding -> Nullable<Vector>,
        bullet_points_embedding -> Nullable<Vector>,
        keywords_embedding -> Nullable<Vector>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

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
        paragraph_description -> Nullable<Text>,
        bullet_points -> Nullable<Text>,
        keywords -> Nullable<Text>,
        paragraph_description_embedding -> Nullable<Vector>,
        bullet_points_embedding -> Nullable<Vector>,
        keywords_embedding -> Nullable<Vector>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use pgvector::sql_types::*;

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
    use pgvector::sql_types::*;

    embeddings (id) {
        id -> Uuid,
        article_id -> Uuid,
        embedding_vector -> Vector,
    }
}

diesel::joinable!(article_chunks -> articles (article_id));
diesel::joinable!(articles -> collections (collection_id));
diesel::joinable!(content_versions -> articles (article_id));
diesel::joinable!(embeddings -> articles (article_id));

diesel::allow_tables_to_appear_in_same_query!(
    article_chunks,
    articles,
    collections,
    content_versions,
    embeddings,
);
