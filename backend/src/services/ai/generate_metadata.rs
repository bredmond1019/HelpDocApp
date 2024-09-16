use super::AIService;
use crate::models::articles::Article;

impl AIService {
    pub async fn generate_article_metadata(
        &self,
        article: &Article,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            "Read the following article and provide:\n
            1. A paragraph description of the article\n
            2. 5-10 bullet points of important facts\n
            3. 5-20 keywords or phrases about the article\n

            You response should be in the following format:\n
            1. Response to part 1\n
            2. Response to part 2\n
            3. Response to part 3\n\n
            Article content:\n{}",
            article
                .markdown_content
                .as_deref()
                .unwrap_or(&article.title)
        );

        let response = self.generate_response(prompt).await?;
        // let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok(response)
    }

    pub async fn generate_collection_metadata(
        &self,
        articles: &Vec<Article>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = articles
            .iter()
            .map(|article| {
                format!(
                    "Article: {}\n
                    Description: {}\n
                    Bullet Points: {}\n
                    Keywords: {}\n",
                    article.title,
                    article.paragraph_description.as_deref().unwrap_or(""),
                    article.bullet_points.as_deref().unwrap_or(""),
                    article.keywords.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "Based on the following metadata from articles in a collection, provide:\n
            1. A paragraph description of the collection\n
            2. 5-10 bullet points summarizing the collection\n
            3. 5-20 keywords or phrases about the collection\n\n
            You response should be in the following format:\n
            1. Response to part 1\n
            2. Response to part 2\n
            3. Response to part 3\n\n
            Collection metadata:\n{}",
            metadata
        );

        let response = self.generate_response(prompt).await?;
        // let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok(response)
    }
}
