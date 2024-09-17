use diesel::prelude::*;
use diesel::ExpressionMethods;

use super::Article;
use crate::schema::articles;
use crate::services::data_processor::ProcessResult;

impl Article {
    pub fn update_metadata(
        &self,
        conn: &mut PgConnection,
        process_result: ProcessResult,
    ) -> Result<(), diesel::result::Error> {
        diesel::update(articles::table.find(self.id))
            .set((
                articles::columns::paragraph_description.eq(process_result.paragraph),
                articles::columns::bullet_points.eq(process_result.bullets),
                articles::columns::keywords.eq(process_result.keywords),
                articles::columns::paragraph_description_embedding
                    .eq(process_result.paragraph_description_embedding),
                articles::columns::bullet_points_embedding
                    .eq(process_result.bullet_points_embedding),
                articles::columns::keywords_embedding.eq(process_result.keywords_embedding),
            ))
            .execute(conn)?;

        Ok(())
    }

    pub fn update_markdown_content(
        &self,
        conn: &mut PgConnection,
        markdown: String,
    ) -> Result<(), diesel::result::Error> {
        use crate::schema::articles::dsl::*;

        diesel::update(articles.find(self.id))
            .set(markdown_content.eq(markdown))
            .execute(conn)?;

        Ok(())
    }
}
