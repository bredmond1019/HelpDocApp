use log::info;

use super::AIService;
use crate::models::articles::Article;

impl AIService {
    pub async fn generate_article_metadata(
        &self,
        article: &Article,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Analyze the following article and provide a structured response EXACTLY as specified below. Follow these instructions precisely:
        
        1. Your response MUST contain these three sections in this order: [SUMMARY], [FACTS], and [KEYWORDS].
        2. Each section MUST be preceded by its header in square brackets.
        3. Do not include any text before [SUMMARY] or after [KEYWORDS].
        4. If you cannot provide content for a section, use "N/A" as the content.
        
        [SUMMARY]
        Provide a concise one-paragraph summary of the article's main points. If unable to summarize, write "N/A".
        
        [FACTS]
        List 5-10 important facts from the article, each on a new line starting with a dash (-). If unable to extract facts, write "N/A".
        
        [KEYWORDS]
        List relevant keywords or phrases, separated by commas, to improve search results. Use 1-2 words per term. If unable to provide keywords, write "N/A".
        
        Article content:
        {}"#,
            article
                .markdown_content
                .as_deref()
                .unwrap_or(&article.title)
        );

        info!("Generating AI response for current prompt");

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
            2. 5-10 bullet points summarizing the collection\n\n
            You response should be in the following format with the following headers
            for each section. I need the headers to be able to parse the response:\n
            1. Response to part 1\n
            2. Response to part 2\n\n
            Collection metadata:\n{}",
            metadata
        );

        let response = self.generate_response(prompt).await?;
        // let (paragraph, bullets, keywords) = self.parse_article_metadata(&response);

        Ok(response)
    }
}
