//! mtbl executor actor
extern crate mtbl;
extern crate serde_cbor;

use actix::prelude::*;
use slog;

use mtbl::Read;
use std;

use logger;

/// This is mtbl executor actor. We are going to run 3 of them in parallel.
pub struct MtblExecutor {
    pub reader: std::sync::Arc<mtbl::Reader>,
    pub logger: slog::Logger,
}

/// This is only message that this actor can handle, but it is easy to extend with more
/// messages.
pub struct GetCountry {
    pub name: String,
}

type MtblResult =
    std::result::Result<Option<serde_cbor::value::Value>, Box<std::error::Error + Send>>;

impl Message for GetCountry {
    type Result = MtblResult;
}

impl Actor for MtblExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GetCountry> for MtblExecutor {
    type Result = MtblResult;

    fn handle(&mut self, msg: GetCountry, _: &mut Self::Context) -> Self::Result {
        let guard = logger::FnGuard::new(
            self.logger.clone(),
            o!("name"=>msg.name.clone()),
            "GetCountry",
        );
        info!(guard, "retrieving country");
        let mr = &self.reader;
        if let Some(ref val) = mr.get(msg.name) {
            let cbor = serde_cbor::from_slice(&val).unwrap();
            return Ok(cbor);
        }
        Ok(None)
    }
}
