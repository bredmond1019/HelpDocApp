#[cfg(test)]
mod tests {
    use backend::models::Article;
    use backend::services::EmbeddingService;
    use diesel::connection::Connection;
    use diesel::pg::PgConnection;
    use dotenv::dotenv;
    use pgvector::Vector;
    use std::env;

    fn establish_connection() -> PgConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }

    #[tokio::test]
    async fn test_find_relevant_articles() {
        let mut conn = establish_connection();
        let embedding_service = EmbeddingService::new();

        // Create a test embedding
        let test_embedding = embedding_service.generate_embedding("How can I create an Organization?").await.unwrap();
        let test_embedding = Vector::from(test_embedding);

        // Call the function
        let results = Article::find_relevant_articles(&test_embedding, &mut conn).await.unwrap();

        // Assert that we got results
        assert!(!results.is_empty(), "No results returned from find_relevant_articles");

        // Check that we got at most 5 results
        assert!(results.len() <= 5, "More than 5 results returned");

        // Check that the similarities are in descending order
        let similarities: Vec<f64> = results.iter().map(|(_, sim)| *sim).collect();
        assert!(similarities.windows(2).all(|w| w[0] >= w[1]), "Results are not in descending order of similarity");

        // Optionally, print out the results for manual inspection
        for (article, similarity) in results {
            println!("Article: {}, Similarity: {}", article.title, similarity);
        }
    }
}