use diesel::prelude::*;
use diesel::ExpressionMethods;
use pgvector::Vector;

use super::Article;
use crate::schema::articles;

impl Article {
    pub fn update_metadata(
        &self,
        conn: &mut PgConnection,
        paragraph_description: String,
        bullet_points: String,
        keywords: String,
        paragraph_description_embedding: Vector,
        bullet_points_embedding: Vector,
        keywords_embedding: Vector,
    ) -> Result<(), diesel::result::Error> {
        diesel::update(articles::table.find(self.id))
            .set((
                articles::columns::paragraph_description.eq(paragraph_description),
                articles::columns::bullet_points.eq(bullet_points),
                articles::columns::keywords.eq(keywords),
                articles::columns::paragraph_description_embedding
                    .eq(paragraph_description_embedding),
                articles::columns::bullet_points_embedding.eq(bullet_points_embedding),
                articles::columns::keywords_embedding.eq(keywords_embedding),
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
