//! sled executor actor

use crate::gql;
use actix::prelude::*;
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
}

impl Actor for SledExecutor {
    type Context = SyncContext<Self>;
}

impl Message for gql::GraphQLData {
    type Result = Result<String, serde_json::Error>;
}

impl Handler<gql::GraphQLData> for SledExecutor {
    type Result = Result<String, serde_json::Error>;

    fn handle(&mut self, msg: gql::GraphQLData, _ctx: &mut Self::Context) -> Self::Result {
        let logger = self.logger.clone();
        info!(self.logger, "{:#?}", msg);
        let res = msg
            .0
            .execute(&self.schema, &gql::GraphQLCtx(self.reader.clone(), logger));
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
