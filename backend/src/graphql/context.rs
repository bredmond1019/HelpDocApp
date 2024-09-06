// File: backend/src/graphql/context.rs

use sqlx::PgPool;
use std::sync::Arc;

pub struct GraphQLContext {
    pub db_pool: Arc<PgPool>,
    // TODO: Add other context fields as needed
}

impl GraphQLContext {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}
