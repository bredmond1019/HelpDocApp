#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use backend::db::vector_db::init_test_vector_db;

    use chrono::Utc;
    use dotenv::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    use backend::{
        data_processing::{generate_embeddings, store_in_postgres},
        models::{Article, Collection},
    };

    use sqlx::PgPool;

    async fn setup_test_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    #[sqlx::test]
    async fn test_store_in_postgres() {
        let pool = setup_test_db().await;

        let collection = NewCollection::new(
            "Test Collection".to_string(),
            Some("Test Description".to_string()),
            "test-collection".to_string(),
        );

        let articles = vec![Article::new(
            1,
            "Test Article".to_string(),
            "test-article".to_string(),
            Some("<p>Test content</p>".to_string()),
        )];

        let result = store_in_postgres(&pool, &collection, &articles).await;
        assert!(result.is_ok());

        // Verify that the data was stored correctly
        let stored_collection = sqlx::query!(
            r#"SELECT id, name, description, slug, created_at, updated_at FROM collections WHERE id = $1"#,
            collection.id,
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch stored collection");

        assert_eq!(stored_collection.name, collection.name);
        assert_eq!(stored_collection.description, collection.description);
        assert_eq!(stored_collection.slug, collection.slug);

        let stored_article = sqlx::query!(
            r#"SELECT id, collection_id, title, slug, html_content, markdown_content, version, last_edited_by, created_at, updated_at FROM articles WHERE id = $1"#,
            articles[0].id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch stored article");

        assert_eq!(stored_article.title, articles[0].title);
        assert_eq!(stored_article.slug, articles[0].slug);
        assert_eq!(stored_article.html_content, articles[0].html_content);
        assert_eq!(
            stored_article.markdown_content,
            articles[0].markdown_content
        );
        assert_eq!(stored_article.version, Some(articles[0].version));
        assert_eq!(stored_article.last_edited_by, articles[0].last_edited_by);
    }

    #[tokio::test]
    async fn test_generate_embeddings() {
        let vector_db = Arc::new(
            init_test_vector_db()
                .await
                .expect("Failed to initialize vector db"),
        );

        // Create test articles
        let articles = vec![
            NewArticle::new(
                1,
                "Test Article 1".to_string(),
                "test-article-1".to_string(),
                Some("<p>Test content 1</p>".to_string()),
                "helpscout_collection_id_1".to_string(),
            ),
            NewArticle::new(
                1,
                "Test Article 2".to_string(),
                "test-article-2".to_string(),
                Some("<p>Test content 2</p>".to_string()),
                "helpscout_collection_id_2".to_string(),
            ),
        ];

        // Call the function under test
        let result = generate_embeddings(&vector_db, &articles).await;

        // Assert the result
        assert!(result.is_ok());
        let embeddings = result.expect("Failed to generate embeddings");
        assert_eq!(embeddings.len(), 2);

        for (i, embedding) in embeddings.iter().enumerate() {
            assert_eq!(embedding.id, articles[i].id);
            assert_eq!(embedding.article_id, articles[i].id);
            assert_eq!(embedding.embedding_vector, vec![0.1, 0.2, 0.3]);
        }
    }
}
