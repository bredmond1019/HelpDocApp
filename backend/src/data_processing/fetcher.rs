// File: src/data_processing/fetcher.rs

use std::env;

use crate::models::{article::NewArticle, collection::NewCollection, Article, Collection};
use anyhow::{anyhow, Result};
use reqwest;
use serde_json::Value;

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let base_url = env::var("API_BASE_URL").expect("API_BASE_URL must be set");
        let api_key = env::var("API_KEY").expect("API_KEY must be set");
        Ok(Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        })
    }

    async fn get(&self, endpoint: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn list_collections(&self, page: Option<i32>) -> Result<Vec<Collection>> {
        let page_param = page.map(|p| format!("&page={}", p)).unwrap_or_default();
        let data = self.get(&format!("/v1/collections?{}", page_param)).await?;
        parse_collections(data)
    }

    pub async fn get_collection(&self, id: &str) -> Result<Collection> {
        let data = self.get(&format!("/v1/collections/{}", id)).await?;
        parse_collection(&data)
    }

    pub async fn list_articles(
        &self,
        collection_id: &str,
        page: Option<i32>,
    ) -> Result<Vec<Article>> {
        let page_param = page.map(|p| format!("&page={}", p)).unwrap_or_default();
        let data = self
            .get(&format!(
                "/v1/collections/{}/articles?{}",
                collection_id, page_param
            ))
            .await?;
        parse_articles(data)
    }

    pub async fn get_article(&self, id: &str) -> Result<Article> {
        let data = self.get(&format!("/v1/articles/{}", id)).await?;
        parse_article(&data)
    }
}

fn parse_collections(data: Value) -> Result<Vec<Collection>> {
    data["collections"]["items"]
        .as_array()
        .ok_or_else(|| anyhow!("Invalid collections data"))?
        .iter()
        .map(parse_collection)
        .collect()
}

fn parse_articles(data: Value) -> Result<Vec<Article>> {
    let collectionId = data["articles"]["collectionId"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid collection id"))?;
    data["articles"]["items"]
        .as_array()
        .ok_or_else(|| anyhow!("Invalid articles data"))?
        .iter()
        .map(|article| parse_article(article, collectionId))
        .collect()
}

fn parse_collection(data: &Value) -> Result<NewCollection> {
    Ok(NewCollection::new(
        data["name"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid collection name"))?
            .to_string(),
        data["description"].as_str().map(|s| s.to_string()),
        data["slug"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid collection slug"))?
            .to_string(),
        data["id"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid collection id"))?
            .to_string(),
    ))
}

fn parse_article(data: &Value, collectionId: ) -> Result<NewArticle> {
    Ok(NewArticle::new(
        collection.id,
        data["title"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid article title"))?
            .to_string(),
        data["slug"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid article slug"))?
            .to_string(),
        data["html_content"].as_str().map(|s| s.to_string()),
        data["collection_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid collection id"))?
            .to_string(),
    ))
}
