// @generated automatically by Diesel CLI.

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
        version -> Nullable<Int4>,
        #[max_length = 255]
        last_edited_by -> Nullable<Varchar>,
        #[max_length = 255]
        helpscout_collection_id -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
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
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
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
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    embeddings (id) {
        id -> Uuid,
        article_id -> Uuid,
        embedding_vector -> Array<Float4>,
    }
}

diesel::joinable!(articles -> collections (collection_id));
diesel::joinable!(content_versions -> articles (article_id));
diesel::joinable!(embeddings -> articles (article_id));

diesel::allow_tables_to_appear_in_same_query!(articles, collections, content_versions, embeddings,);
