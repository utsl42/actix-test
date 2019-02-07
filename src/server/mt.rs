//! mtbl executor actor

use actix::prelude::*;
use juniper;
use juniper::http::GraphQLRequest;
use juniper::GraphQLObject;
use mtbl::Read;
use serde_cbor;
use serde_derive::{Deserialize, Serialize};
use slog::{info, error, o};
use std;

use crate::logger;

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "A country record")]
pub struct Country {
    name: CountryName,
    tld: Vec<String>,
    cca2: String,
    ccn3: String,
    cca3: String,
    cioc: String,
    independent: bool,
    currency: Vec<String>,
    calling_code: Vec<String>,
    capital: Vec<String>,
    region: String,
    subregion: String,
    latlng: Vec<f64>,
    borders: Vec<String>,
    area: f64,
    flag: String,
}

#[derive(GraphQLObject, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "A country record")]
pub struct CountryName {
    common: String,
    official: String,
}

pub struct QueryRoot;

juniper::graphql_object!(QueryRoot: GraphQLCtx |&self| {
    field country(&executor, name: String) -> juniper::FieldResult<Option<Country>> {
        let ctx = executor.context();
        if let Some(ref val) = ctx.get(name) {
            let res = serde_cbor::from_slice(&val);
            if res.is_ok() {
                Ok(res.unwrap())
            } else {
                error!(ctx.logger(), "error decoding CBOR: {:#?}", res);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
});

pub type Schema = juniper::RootNode<'static, QueryRoot, juniper::EmptyMutation<GraphQLCtx>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, juniper::EmptyMutation::<GraphQLCtx>::new())
}

/// This is mtbl executor actor. We are going to run 3 of them in parallel.
pub struct MtblExecutor {
    pub reader: std::sync::Arc<mtbl::Reader>,
    pub logger: slog::Logger,
    pub schema: std::sync::Arc<Schema>,
}

impl MtblExecutor {
    pub fn new(reader: std::sync::Arc<mtbl::Reader>, logger: slog::Logger) -> MtblExecutor {
        MtblExecutor {
            reader,
            logger,
            schema: std::sync::Arc::new(create_schema()),
        }
    }
}

/// This is only message that this actor can handle, but it is easy to extend with more
/// messages.
pub struct GetCountry {
    pub name: String,
}

type MtblResult = std::result::Result<Option<serde_cbor::value::Value>, serde_cbor::error::Error>;

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
            serde_cbor::from_slice(&val)
        } else {
            Ok(None)
        }
    }
}

pub struct GraphQLCtx(std::sync::Arc<mtbl::Reader>, slog::Logger);
impl GraphQLCtx {
    fn get(&self, name: String) -> std::option::Option<std::vec::Vec<u8>> {
        self.0.get(name)
    }

    fn logger(&self) -> &slog::Logger {
        &self.1
    }
}
impl juniper::Context for GraphQLCtx {}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, serde_json::Error>;
}

impl Handler<GraphQLData> for MtblExecutor {
    type Result = Result<String, serde_json::Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg
            .0
            .execute(&self.schema, &GraphQLCtx(self.reader.clone(), self.logger.clone()));
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
