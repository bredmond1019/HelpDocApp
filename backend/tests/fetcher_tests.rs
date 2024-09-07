#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dotenv::dotenv;
    use mockito::Server;
    use serde_json::json;
    use tokio;

    use backend::data_processing::fetcher::ApiClient;
    use backend::models::Collection;

    #[tokio::test]
    async fn test_parse_collection() -> Result<()> {
        dotenv().ok();
        let mut server = Server::new_async().await;

        let _m = server
            .mock("GET", "/v1/collections/5214c83d45667acd25394b53")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "collection": {
                        "id": "5214c83d45667acd25394b53",
                        "siteId": "52404efc4566740003092640",
                        "number": 33,
                        "slug": "my-collection",
                        "visibility": "public",
                        "order": 1,
                        "name": "My Collection",
                        "description": "Description of my collection",
                        "publicUrl": "https://my-docs.helpscoutdocs.com/collection/1-test",
                        "articleCount": 3,
                        "publishedArticleCount": 1,
                        "createdBy": 73423,
                        "updatedBy": 73423,
                        "createdAt": "2013-08-21T14:01:33Z",
                        "updatedAt": "2013-08-21T14:01:33Z"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let base_url = server.url();
        let api_client = ApiClient::new(Some(base_url), Some("test_api_key".to_string()))?;

        let collection = api_client
            .get_collection("5214c83d45667acd25394b53")
            .await?;

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

        let _m = server
            .mock("GET", "/v1/articles/521632244566c845e582652d")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "article": {
                        "id": "521632244566c845e582652d",
                        "number": 125,
                        "collectionId": "5214c77c45667acd25394b51",
                        "slug": "my-article",
                        "status": "published",
                        "hasDraft": false,
                        "name": "My Article",
                        "text": "This is the text of the article.",
                        "categories": [
                        "5214c77d45667acd25394b52"
                        ],
                        "related": [
                        "521632244566c845e582652b",
                        "521632244566c845e582652c"
                        ],
                        "publicUrl": "https://docs.helpscout.net/article/100-my-article",
                        "popularity": 4.3,
                        "viewCount": 237,
                        "createdBy": 73423,
                        "updatedBy": 73423,
                        "createdAt": "2013-08-22T15:45:40Z",
                        "updatedAt": "2013-08-22T21:40:56Z",
                        "lastPublishedAt": "2013-08-22T21:40:56Z"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let base_url = server.url();
        let api_client = ApiClient::new(Some(base_url), Some("test_api_key".to_string()))?;

        let collection = Collection::new(
            "5214c77c45667acd25394b51".to_string(),
            "Test Collection".to_string(),
            None,
            "test-collection".to_string(),
        );

        let article = api_client
            .get_article("521632244566c845e582652d", &collection)
            .await?;

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

        let _m = server
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
                                "siteId": "52404efc4566740003092640",
                                "number": 33,
                                "slug": "my-collection",
                                "visibility": "public",
                                "order": 1,
                                "name": "My Collection",
                                "description": "Description of my collection",
                                "publicUrl": "https://my-docs.helpscoutdocs.com/collection/1-test",
                                "articleCount": 3,
                                "publishedArticleCount": 1,
                                "createdBy": 73423,
                                "updatedBy": 73423,
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

        let base_url = server.url();
        let api_client = ApiClient::new(Some(base_url), Some("test_api_key".to_string()))?;

        let collections = api_client.get_list_collections().await?;

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

        let _m = server
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
                                "number": 121,
                                "collectionId": "5214c77c45667acd25394b51",
                                "status": "published",
                                "hasDraft": false,
                                "name": "My Article",
                                "publicUrl": "https://docs.helpscout.net/article/100-my-article",
                                "popularity": 4.3,
                                "viewCount": 237,
                                "createdBy": 73423,
                                "updatedBy": null,
                                "createdAt": "2013-08-21T19:34:13Z",
                                "updatedAt": null,
                                "lastPublishedAt": "2013-08-21T19:34:13Z"
                            }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let base_url = server.url();
        let api_client = ApiClient::new(Some(base_url), Some("test_api_key".to_string()))?;

        let collection = Collection::new(
            "5214c77c45667acd25394b51".to_string(),
            "Test Collection".to_string(),
            None,
            "test-collection".to_string(),
        );

        let article_refs = api_client.get_list_articles(&collection).await?;

        assert_eq!(article_refs.len(), 1);
        assert_eq!(article_refs[0].name, "My Article");
        assert_eq!(article_refs[0].id, "5215163545667acd25394b5c");
        assert_eq!(article_refs[0].collection_id, "5214c77c45667acd25394b51");

        Ok(())
    }
}
