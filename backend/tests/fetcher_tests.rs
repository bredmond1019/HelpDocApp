#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use tokio;

    use anyhow::Result;
    use backend::models::{
        article::{ArticleJson, NewArticle},
        collection::NewCollection,
        Article, Collection,
    };
    use mockito::Server;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_parse_collection() -> Result<()> {
        dotenv().ok();
        let mut server = Server::new_async().await;

        let m = server
            .mock("GET", "/v1/collections/5214c83d45667acd25394b53")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "collection": {
                        "id": "5214c83d45667acd25394b53",
                        "name": "My Collection",
                        "description": "Description of my collection",
                        "slug": "my-collection",
                        "createdAt": "2013-08-21T14:01:33Z",
                        "updatedAt": "2013-08-21T14:01:33Z"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let base_url = server.url();

        let response = client
            .get(format!(
                "{}/v1/collections/5214c83d45667acd25394b53",
                base_url
            ))
            .header("Authorization", "Bearer 1234567890")
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get response text");

        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        let collection: NewCollection = serde_json::from_value(json_value["collection"].clone())?;

        assert_eq!(collection.name, "My Collection");
        assert_eq!(
            collection.description,
            Some("Description of my collection".to_string())
        );
        assert_eq!(collection.slug, "my-collection");

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_article() -> Result<()> {
        dotenv().ok();
        let mut server = Server::new_async().await;

        let m = server
            .mock("GET", "/v1/articles/521632244566c845e582652d")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "article": {
                        "id": "521632244566c845e582652d",
                        "collectionId": "5214c77c45667acd25394b51",
                        "title": "My Article",
                        "slug": "my-article",
                        "text": "This is the text of the article.",
                        "createdAt": "2013-08-22T15:45:40Z",
                        "updatedAt": "2013-08-22T21:40:56Z"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let base_url = server.url();

        let response = client
            .get(format!("{}/v1/articles/521632244566c845e582652d", base_url))
            .header("Authorization", "Bearer 1234567890")
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get response text");

        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        let article: NewArticle = serde_json::from_value(json_value["article"].clone())?;

        assert_eq!(article.title, "My Article");
        assert_eq!(article.slug, "my-article");
        assert_eq!(
            article.html_content,
            Some("This is the text of the article.".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_list_collections() -> Result<()> {
        dotenv().ok();
        let mut server = Server::new_async().await;

        let m = server
            .mock("GET", "/v1/collections")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "collections": {
                        "page": 1,
                        "pages": 1,
                        "count": 1,
                        "items": [
                            {
                                "id": "5214c83d45667acd25394b53",
                                "name": "My Collection",
                                "description": "Description of my collection",
                                "slug": "my-collection",
                                "createdAt": "2013-08-21T14:01:33Z",
                                "updatedAt": "2013-08-21T14:01:33Z"
                            }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let base_url = server.url();

        let response = client
            .get(format!("{}/v1/collections", base_url))
            .header("Authorization", "Bearer 1234567890")
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get response text");

        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        let collections: Vec<NewCollection> =
            serde_json::from_value(json_value["collections"]["items"].clone())?;

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].name, "My Collection");
        assert_eq!(
            collections[0].description,
            Some("Description of my collection".to_string())
        );
        assert_eq!(collections[0].slug, "my-collection");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_articles() -> Result<()> {
        dotenv().ok();
        let mut server = Server::new_async().await;

        let m = server
            .mock("GET", "/v1/collections/5214c77c45667acd25394b51/articles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "articles": {
                        "page": 1,
                        "pages": 1,
                        "count": 1,
                        "items": [
                            {
                                "id": "5215163545667acd25394b5c",
                                "collectionId": "5214c77c45667acd25394b51",
                                "title": "My Article",
                                "slug": "my-article",
                                "createdAt": "2013-08-21T19:34:13Z",
                                "updatedAt": "2013-08-21T19:34:13Z",
                                "lastPublishedAt": "2013-08-21T19:34:13Z"
                            }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let base_url = server.url();

        let response = client
            .get(format!(
                "{}/v1/collections/5214c77c45667acd25394b51/articles",
                base_url
            ))
            .header("Authorization", "Bearer 1234567890")
            .send()
            .await
            .expect("Failed to get response")
            .text()
            .await
            .expect("Failed to get response text");

        println!("{:?}", response);

        let json_value: serde_json::Value = serde_json::from_str(&response)?;
        let articles: Vec<ArticleJson> =
            serde_json::from_value(json_value["articles"]["items"].clone())?;

        println!("ARTICLES: {:?}", articles);

        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].title, "My Article");
        assert_eq!(articles[0].slug, "my-article");
        assert_eq!(articles[0].collection_id, "5214c77c45667acd25394b51");

        // If you need to convert ArticleJson to NewArticle or Article, you can do it here
        let new_articles: Vec<NewArticle> = articles
            .into_iter()
            .map(|article| NewArticle {
                helpscout_collection_id: article.collection_id.clone(),
                collection_id: Uuid::new_v4(), // Generate a new UUID for testing purposes
                title: article.title,
                slug: article.slug,
                html_content: None, // You might want to fetch this separately if needed
            })
            .collect();

        assert_eq!(new_articles.len(), 1);
        assert_eq!(new_articles[0].title, "My Article");
        assert_eq!(new_articles[0].slug, "my-article");

        Ok(())
    }
}
