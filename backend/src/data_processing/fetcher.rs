// File: src/data_processing/fetcher.rs

use anyhow::Result;
use reqwest;
use serde_json::{from_value, Value};
use std::env;

use crate::models::{
    article::{ArticleFull, ArticleFullResponse, ArticleRef, ArticleResponse},
    collection::{CollectionItem, CollectionResponse},
    Article, Collection,
};

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

    pub async fn get_list_collections(&self) -> Result<Vec<Collection>> {
        let data = self.get("/v1/collections").await?;
        let collection_response: CollectionResponse = from_value(data)?;
        let collection_data = collection_response.collection_data;

        let mut collections = Vec::new();

        collections.extend(parse_collections(collection_data.collections)?);

        // let pages = collection_data.pages;
        // for page in 1..=pages {
        //     let data = self.get(&format!("/v1/collections?page={}", page)).await?;
        //     collections.extend(parse_collections(data)?);
        // }
        Ok(collections)
    }

    pub async fn get_collection(&self, id: &str) -> Result<Collection> {
        let data = self.get(&format!("/v1/collections/{}", id)).await?;
        let collection_item: CollectionItem = from_value(data)?;
        parse_collection(&collection_item)
    }

    pub async fn get_list_articles(&self, collection: &Collection) -> Result<Vec<ArticleRef>> {
        let data = self
            .get(&format!(
                "/v1/collections/{}/articles",
                collection.helpscout_collection_id
            ))
            .await?;

        let api_response: ArticleResponse = from_value(data)?;
        let article_data = api_response.article_data;

        let mut articles_refs: Vec<ArticleRef> = Vec::new();

        articles_refs.extend(article_data.article_refs);

        // let pages = article_data.pages;
        // for page in 1..=pages {
        //     let data = self
        //         .get(&format!(
        //             "/v1/collections/{}/articles?page={}",
        //             helpscout_collection_id, page
        //         ))
        //         .await?;
        //     articles.extend(parse_articles(data)?);
        // }
        Ok(articles_refs)
    }

    pub async fn get_article(&self, id: &str, collection: &Collection) -> Result<Article> {
        let data = self.get(&format!("/v1/articles/{}", id)).await?;
        let api_response: ArticleFullResponse = from_value(data)?;
        let article = api_response.article;
        parse_article(&article, collection)
    }
}

fn parse_collections(collections: Vec<CollectionItem>) -> Result<Vec<Collection>> {
    collections.iter().map(parse_collection).collect()
}

fn parse_collection(collection: &CollectionItem) -> Result<Collection> {
    Ok(Collection::new(
        collection.id.clone(),
        collection.name.clone(),
        collection.description.clone(),
        collection.slug.clone(),
    ))
}

fn parse_article(helpscout_article: &ArticleFull, collection: &Collection) -> Result<Article> {
    let article = Article::new(
        collection.id,
        helpscout_article.collection_id.clone(),
        helpscout_article.id.clone(),
        helpscout_article.name.clone(),
        helpscout_article.slug.clone(),
        Some(helpscout_article.text.clone()),
    );
    Ok(article)
}
