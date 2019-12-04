//! sled executor actor

use crate::gql;
use slog::info;
use std;

/// This is sled executor actor. We are going to run 3 of them in parallel.
pub struct SledExecutor {
    pub reader: std::sync::Arc<sled::Tree>,
    pub logger: slog::Logger,
    pub schema: std::sync::Arc<gql::Schema>,
}


impl SledExecutor {
    pub fn new(reader: std::sync::Arc<sled::Tree>, logger: slog::Logger) -> SledExecutor {
        SledExecutor {
            reader,
            logger,
            schema: std::sync::Arc::new(gql::create_schema()),
        }
    }

    pub fn handle(&self, msg: gql::GraphQLData) -> std::result::Result<String, serde_json::Error> {
        let logger = self.logger.clone();
        info!(self.logger, "{:#?}", msg);
        let res = msg
            .0
            .execute(&self.schema, &gql::GraphQLCtx(self.reader.clone(), logger));
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
