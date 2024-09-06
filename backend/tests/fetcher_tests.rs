use anyhow::Result;
use backend::data_processing::fetcher::ApiClient;

use mockito::Server;
use serde_json::json;

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_parse_collection() -> Result<()> {
        let mut server = Server::new();

        let _m = server
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
            .create();

        let client = ApiClient::new()?;
        let collection = client.get_collection("5214c83d45667acd25394b53").await?;

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
        let mut server = Server::new();

        let _m = server
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
            .create();

        let client = ApiClient::new()?;
        let article = client.get_article("521632244566c845e582652d").await?;

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
        let mut server = Server::new();

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
            .create();

        let client = ApiClient::new()?;
        let collections = client.list_collections(None).await?;

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
        let mut server = Server::new();

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
                                "collectionId": "5214c77c45667acd25394b51",
                                "title": "My Article",
                                "slug": "my-article",
                                "createdAt": "2013-08-21T19:34:13Z",
                                "updatedAt": null,
                                "lastPublishedAt": "2013-08-21T19:34:13Z"
                            }
                        ]
                    }
                })
                .to_string(),
            )
            .create();

        let client = ApiClient::new()?;
        let articles = client
            .list_articles("5214c77c45667acd25394b51", None)
            .await?;

        assert_eq!(articles.len(), 1);
        assert_eq!(articles[0].title, "My Article");
        assert_eq!(articles[0].slug, "my-article");

        Ok(())
    }
}
