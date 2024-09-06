#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    // ... other imports and mocks ...

    mock! {
        SentenceEmbeddings {}
        trait SentenceEmbeddings {
            fn encode(&self, input: &[&str]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>>;
        }
    }

    mock! {
        SentenceEmbeddingsBuilder {}
        impl SentenceEmbeddingsBuilder {
            fn remote(model_type: SentenceEmbeddingsModelType) -> Self;
            fn create_model(&self) -> Result<MockSentenceEmbeddings, Box<dyn std::error::Error>>;
        }
    }

    #[tokio::test]
    async fn test_generate_embeddings() {
        let mut mock_qdrant = MockQdrant::new();
        mock_qdrant.expect_upsert_points().returning(|_| Ok(()));

        let mut mock_sentence_embeddings = MockSentenceEmbeddings::new();
        mock_sentence_embeddings
            .expect_encode()
            .returning(|_| Ok(vec![vec![0.1, 0.2, 0.3]]));

        let mut mock_sentence_embeddings_builder = MockSentenceEmbeddingsBuilder::new();
        mock_sentence_embeddings_builder
            .expect_remote()
            .return_const(MockSentenceEmbeddingsBuilder::new());
        mock_sentence_embeddings_builder
            .expect_create_model()
            .returning(move || Ok(mock_sentence_embeddings.clone()));

        // Replace the actual SentenceEmbeddingsBuilder with the mock in your generate_embeddings function
        // You might need to refactor generate_embeddings to accept a SentenceEmbeddingsBuilder as a parameter

        let articles = vec![Article {
            id: 1,
            collection_id: 1,
            title: "Test Article".to_string(),
            slug: "test-article".to_string(),
            html_content: Some("<p>Test content</p>".to_string()),
            markdown_content: Some("Test content".to_string()),
            version: 1,
            last_edited_by: Some("Test User".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let result =
            generate_embeddings(&mock_qdrant, &articles, mock_sentence_embeddings_builder).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].article_id, 1);
        assert!(!embeddings[0].embedding_vector.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use dotenv::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    #[sqlx::test]
    async fn test_store_in_postgres() {
        let pool = setup_test_db().await;

        let collection = Collection {
            id: 1,
            name: "Test Collection".to_string(),
            description: Some("Test Description".to_string()),
            slug: "test-collection".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let articles = vec![Article {
            id: 1,
            collection_id: 1,
            title: "Test Article".to_string(),
            slug: "test-article".to_string(),
            html_content: Some("<p>Test content</p>".to_string()),
            markdown_content: Some("Test content".to_string()),
            version: 1,
            last_edited_by: Some("Test User".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let result = store_in_postgres(&pool, &collection, &articles).await;
        assert!(result.is_ok());

        // Verify that the data was stored correctly
        let stored_collection = sqlx::query_as!(
            Collection,
            "SELECT * FROM collections WHERE id = $1",
            collection.id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch stored collection");

        assert_eq!(stored_collection.name, collection.name);

        let stored_article = sqlx::query_as!(
            Article,
            "SELECT * FROM articles WHERE id = $1",
            articles[0].id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch stored article");

        assert_eq!(stored_article.title, articles[0].title);
    }

    #[tokio::test]
    async fn test_generate_embeddings() {
        let mut mock_qdrant = MockQdrant::new();
        mock_qdrant.expect_upsert_points().returning(|_| Ok(()));

        let mut mock_sentence_embeddings = MockSentenceEmbeddings::new();
        mock_sentence_embeddings
            .expect_encode()
            .returning(|_| Ok(vec![vec![0.1, 0.2, 0.3]]));

        let mut mock_sentence_embeddings_builder = MockSentenceEmbeddingsBuilder::new();
        mock_sentence_embeddings_builder
            .expect_remote()
            .return_const(MockSentenceEmbeddingsBuilder::new());
        mock_sentence_embeddings_builder
            .expect_create_model()
            .returning(move || Ok(mock_sentence_embeddings.clone()));

        let articles = vec![Article {
            id: 1,
            collection_id: 1,
            title: "Test Article".to_string(),
            slug: "test-article".to_string(),
            html_content: Some("<p>Test content</p>".to_string()),
            markdown_content: Some("Test content".to_string()),
            version: 1,
            last_edited_by: Some("Test User".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let result =
            generate_embeddings(&mock_qdrant, &articles, mock_sentence_embeddings_builder).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].article_id, 1);
        assert!(!embeddings[0].embedding_vector.is_empty());
    }
}
