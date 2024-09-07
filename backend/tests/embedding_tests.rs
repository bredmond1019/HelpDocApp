#[cfg(test)]
mod tests {
    use backend::db::vector_db::init_test_vector_db;
    use backend::{data_processing::generate_embeddings, models::Article};
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_generate_embeddings() {
        let vector_db = Arc::new(
            init_test_vector_db()
                .await
                .expect("Failed to initialize vector db"),
        );

        // Create test articles
        let articles = vec![
            Article::new(
                Uuid::new_v4(),
                "helpscout_collection_id_1".to_string(),
                "helpscout_article_id_1".to_string(),
                "Test Article 1".to_string(),
                "test-article-1".to_string(),
                Some("<p>Test content 1</p>".to_string()),
            ),
            Article::new(
                Uuid::new_v4(),
                "helpscout_collection_id_2".to_string(),
                "helpscout_article_id_2".to_string(),
                "Test Article 2".to_string(),
                "test-article-2".to_string(),
                Some("<p>Test content 2</p>".to_string()),
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
