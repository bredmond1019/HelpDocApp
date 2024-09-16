use diesel::prelude::*;
use uuid::Uuid;

use super::Article;
use crate::schema::article_chunks;

#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::article_chunks)]
pub struct ArticleChunk {
    pub id: Uuid,
    pub article_id: Uuid,
    pub content: String,
    pub is_title: bool,
    pub embedding_id: Option<Uuid>,
}

impl ArticleChunk {
    pub fn store(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        let chunk: Self = diesel::insert_into(article_chunks::table)
            .values(self)
            .get_result(conn)?;
        Ok(chunk)
    }
}

impl Article {
    pub fn create_chunks(&self, chunk_size: usize) -> Vec<ArticleChunk> {
        let mut chunks = Vec::new();

        // Add title as a separate chunk
        let title_chunk = ArticleChunk {
            id: Uuid::new_v4(),
            article_id: self.id,
            content: self.title.clone(),
            is_title: true,
            embedding_id: None,
        };
        chunks.push(title_chunk);

        if let Some(content) = &self.markdown_content {
            // Split content into words
            let words: Vec<&str> = content.split_whitespace().collect();
            let mut current_chunk = String::new();
            let mut word_count = 0;

            for word in words {
                if word_count >= chunk_size {
                    // If the current chunk has reached or exceeded the chunk size,
                    // add it to the chunks vector and start a new chunk
                    chunks.push(ArticleChunk {
                        id: Uuid::new_v4(),
                        article_id: self.id,
                        content: current_chunk.trim().to_string(),
                        is_title: false,
                        embedding_id: None,
                    });
                    current_chunk.clear();
                    word_count = 0;
                }

                // Add the word to the current chunk
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(word);
                word_count += 1;
            }

            // Add any remaining content as the last chunk
            if !current_chunk.is_empty() {
                chunks.push(ArticleChunk {
                    id: Uuid::new_v4(),
                    article_id: self.id,
                    content: current_chunk.trim().to_string(),
                    is_title: false,
                    embedding_id: None,
                });
            }
        }

        chunks
    }
}
