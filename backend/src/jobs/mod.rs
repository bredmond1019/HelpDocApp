use crate::data_processor::data_processor::DataProcessor;
use crate::models::Article;
use crate::models::{article::ArticleRef, Collection};

use anyhow::Result;
use uuid::Uuid;

pub mod enqueue;
pub mod job_queue;
pub mod worker;

pub use job_queue::JobQueue;

pub enum Job {
    SyncCollection(Collection),
    StoreCollection(Collection),
    SyncArticle(ArticleRef, Collection),
    StoreArticle(Article),
    ConvertHtmlToMarkdown(Article),
    GenerateEmbeddings(Article),
}

impl Job {
    async fn process(
        &self,
        processor: &DataProcessor,
    ) -> Result<(Uuid, Result<(), anyhow::Error>), anyhow::Error> {
        match self {
            Job::SyncCollection(collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.prepare_sync_collection(collection).await;
                Ok((job_id, result.map(|_| ())))
            }
            Job::StoreCollection(collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.sync_collection(collection).await;
                Ok((job_id, result))
            }
            Job::SyncArticle(article_ref, collection) => {
                let job_id = Uuid::new_v4();
                let result = processor.sync_article(article_ref, collection).await;
                Ok((job_id, result.map(|_| ())))
            }
            Job::StoreArticle(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.store_article(article).await;
                Ok((job_id, result))
            }
            Job::ConvertHtmlToMarkdown(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.convert_html_to_markdown(article).await;
                Ok((job_id, result))
            }
            Job::GenerateEmbeddings(article) => {
                let job_id = Uuid::new_v4();
                let result = processor.generate_article_embeddings(&article).await;
                Ok((job_id, result))
            }
        }
    }
}
