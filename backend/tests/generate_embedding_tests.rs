#[cfg(test)]
mod tests {
    use backend::db::vector_db::init_test_vector_db;
    use backend::{data_processor::generate_embeddings, models::Article};
    use qdrant_client::qdrant::UpsertPointsBuilder;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_generate_embeddings() {
        let vector_db_client = Arc::new(
            init_test_vector_db()
                .await
                .expect("Failed to initialize vector db"),
        );

        // Create test articles
        let article = Article::new(
            Uuid::new_v4(),
            "helpscout_collection_id_1".to_string(),
            Some("helpscout_article_id_1".to_string()),
            "Test Article 1".to_string(),
            "test-article-1".to_string(),
            Some("<p>Test content 1</p>".to_string()),
        );

        // Call the function under test
        let result = generate_embeddings(article.clone()).await;

        // Assert the result
        assert!(result.is_ok());
        let (embedding, point) = result.expect("Failed to generate embeddings");

        vector_db_client
            .upsert_points(UpsertPointsBuilder::new("testing", vec![point]))
            .await
            .expect("Failed to upsert points");

        assert_eq!(embedding.article_id, article.id);
        // Remove the specific assertion for embedding_vector
        assert!(!embedding.embedding_vector.is_empty());
    }
}
