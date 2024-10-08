// File: src/data_processing/fetcher.rs

use anyhow::Result;
use log::{error, info};
use reqwest;
use serde_json::{from_value, Value};
use std::{env, time::Duration};
use tokio::time::sleep;

use crate::models::{
    articles::{ArticleFull, ArticleFullResponse, ArticleRef, ArticleResponse},
    collection::{CollectionItem, CollectionResponse},
    Article, Collection,
};

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: Option<String>, api_key: Option<String>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()?;
        let api_key = api_key.unwrap_or(env::var("API_KEY").expect("API_KEY must be set"));
        let base_url =
            base_url.unwrap_or(env::var("API_BASE_URL").expect("API_BASE_URL must be set"));
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    async fn get(&self, endpoint: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, endpoint);

        let response = self
            .client
            .get(&url)
            .basic_auth(self.api_key.clone(), Some("DUMMY_PASSWORD"))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request: {:?}", e);
                e
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            error!("Failed to get response body: {:?}", e);
            e
        })?;

        if !status.is_success() {
            error!("API request failed. Status: {}, Body: {}", status, body);
            return Err(anyhow::anyhow!("API request failed: {}", status));
        }

        serde_json::from_str(&body).map_err(|e| {
            error!("Failed to parse JSON: {:?}", e);
            e.into()
        })
    }

    pub async fn get_list_collections(&self) -> Result<Vec<Collection>> {
        info!("Fetching collections from API");
        let data = self.get("/v1/collections").await?;
        info!("API Response | List Collections: {:?}", data);
        let collection_response: CollectionResponse = from_value(data)?;
        info!(
            "API Response | List Collections: {:?}",
            collection_response.collections.items.len()
        );
        let collection_data = collection_response.collections;
        info!("Found {:?} collections", collection_data.items.len());

        let mut collections: Vec<Collection> = Vec::new();

        collections.extend(parse_collections(collection_data.items)?);

        Ok(collections)
    }

    pub async fn get_collection(&self, id: &str) -> Result<Collection> {
        let data = self.get(&format!("/v1/collections/{}", id)).await?;
        let collection_item: CollectionItem = from_value(data["collection"].clone())?;
        info!("API Response | Get Collection: {:?}", collection_item.id);
        parse_collection(&collection_item)
    }

    pub async fn get_list_articles(&self, collection: &Collection) -> Result<Vec<ArticleRef>> {
        let helpscout_collection_id = &collection.helpscout_collection_id;
        let mut articles_refs = Vec::new();
        let mut page = 1;

        loop {
            info!("Fetching articles from page: {}", page);
            let endpoint = format!(
                "/v1/collections/{}/articles?page={}",
                helpscout_collection_id, page
            );
            info!("Sending request to endpoint: {}", endpoint);
            let data = self.get(&endpoint).await?;
            info!("Received response for page: {}", page);
            // info!("API Response | Get List Articles: {:?}", data);
            info!("Attempting to deserialize API response");
            let api_response: ArticleResponse = match from_value(data.clone()) {
                Ok(response) => {
                    info!("Successfully deserialized API response");
                    response
                }
                Err(e) => {
                    error!("Failed to deserialize API response: {:?}", e);
                    // error!("Raw API response: {:?}", data);
                    return Err(anyhow::anyhow!("Failed to deserialize API response: {}", e));
                }
            };
            let article_data = api_response.articles;
            info!(
                "Total pages: {}, Current page: {}",
                article_data.pages, page
            );

            info!(
                "Found {} articles on page {} for collection: {}",
                article_data.items.len(),
                page,
                collection.slug
            );
            articles_refs.extend(article_data.items);

            if page >= article_data.pages {
                break;
            }
            page += 1;
            sleep(Duration::from_millis(1000)).await;
        }

        info!("Total articles fetched: {}", articles_refs.len());
        Ok(articles_refs)
    }

    pub async fn get_article(&self, id: &str, collection: &Collection) -> Result<Article> {
        let data = self.get(&format!("/v1/articles/{}", id)).await?;
        let api_response: ArticleFullResponse = from_value(data)?;
        info!("API Response | Get Article: {:?}", api_response.article.id);
        let article = api_response.article;
        info!(
            "Found article: ID:{:?}, Title: {:?}",
            article.id, article.name
        );

        parse_article(&article, collection)
    }
}

fn parse_collections(collections: Vec<CollectionItem>) -> Result<Vec<Collection>> {
    collections.iter().map(parse_collection).collect()
}

fn parse_collection(collection: &CollectionItem) -> Result<Collection> {
    Ok(Collection::new(
        collection.name.clone(),
        collection.description.clone(),
        collection.slug.clone(),
        collection.id.clone(),
    ))
}

fn parse_article(helpscout_article: &ArticleFull, collection: &Collection) -> Result<Article> {
    let article = Article::new(
        collection.id,
        helpscout_article.collection_id.clone(),
        Some(helpscout_article.id.clone()),
        helpscout_article.name.clone(),
        helpscout_article.slug.clone(),
        Some(helpscout_article.text.clone()),
    );
    Ok(article)
}
