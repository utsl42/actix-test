//! sled executor actor

use actix::prelude::*;
use juniper;
use juniper::http::GraphQLRequest;
use juniper::GraphQLObject;
use serde_cbor;
use serde_derive::{Deserialize, Serialize};
use slog::{error, info, o};
use std;

use crate::logger;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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

juniper::graphql_object!(Country: GraphQLCtx |&self| {
    description: "A country record"

    field name() -> &CountryName {
        &self.name
    }

    field tld() -> &Vec<String> {
        &self.tld
    }

    field cca2() -> &str {
        self.cca2.as_str()
    }

    field ccn3() -> &str {
        self.ccn3.as_str()
    }

    field cca3() -> &str {
        self.cca3.as_str()
    }

    field cioc() -> &str {
        self.cioc.as_str()
    }

    field independent() -> bool {
        self.independent
    }

    field currency() -> &Vec<String> {
        &self.currency
    }

    field calling_code() -> &Vec<String> {
        &self.calling_code
    }

    field capital() -> &Vec<String> {
        &self.capital
    }

    field region() -> &str {
        self.region.as_str()
    }

    field subregion() -> &str {
        self.subregion.as_str()
    }

    field latlng() -> &Vec<f64> {
        &self.latlng
    }

    field flag() -> &str {
        self.flag.as_str()
    }

    field area() -> f64 {
        self.area
    }

    field borders(&executor) -> juniper::FieldResult<Vec<Country>> {
        let ctx = executor.context();
        let countries: Vec<Country> = self.borders.iter()
            .map(|c| ctx.get(c.to_string()))
             .filter_map(|maybe| {
                maybe.and_then( | v | {
                    serde_cbor::from_slice(&v).ok()
                })
            })
            .collect();
        Ok(countries)
    }
});

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

    field list_countries(&executor) -> juniper::FieldResult<Vec<Country>> {
        let ctx = executor.context();
        let results = ctx.iter();
        let countries: Vec<Country> = results
             .filter_map(|maybe| {
                maybe.ok().and_then( | (k, v) | {
                    serde_cbor::from_slice(&v).ok()
                })
            })
            .collect();
        Ok(countries)
    }
});

pub type Schema = juniper::RootNode<'static, QueryRoot, juniper::EmptyMutation<GraphQLCtx>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, juniper::EmptyMutation::<GraphQLCtx>::new())
}

/// This is sled executor actor. We are going to run 3 of them in parallel.
pub struct SledExecutor {
    pub reader: std::sync::Arc<sled::Tree>,
    pub logger: slog::Logger,
    pub schema: std::sync::Arc<Schema>,
}

impl SledExecutor {
    pub fn new(reader: std::sync::Arc<sled::Tree>, logger: slog::Logger) -> SledExecutor {
        SledExecutor {
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

type SledResult = std::result::Result<Option<serde_cbor::value::Value>, serde_cbor::error::Error>;

impl Message for GetCountry {
    type Result = SledResult;
}

impl Actor for SledExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GetCountry> for SledExecutor {
    type Result = SledResult;

    fn handle(&mut self, msg: GetCountry, _: &mut Self::Context) -> Self::Result {
        let guard = logger::FnGuard::new(
            self.logger.clone(),
            o!("name"=>msg.name.clone()),
            "GetCountry",
        );
        info!(guard, "retrieving country");
        let ctx = &GraphQLCtx(self.reader.clone(), self.logger.clone());
        if let Some(ref val) = ctx.get(msg.name) {
            serde_cbor::from_slice(&val)
        } else {
            Ok(None)
        }
    }
}

pub struct GraphQLCtx(std::sync::Arc<sled::Tree>, slog::Logger);
impl GraphQLCtx {
    fn get(&self, name: String) -> std::option::Option<std::vec::Vec<u8>> {
        self.0.get(name).ok()?.and_then(|val| Some(val.to_vec()))
    }

    fn iter(&self) -> sled::Iter {
        self.0.iter()
    }

    fn logger(&self) -> &slog::Logger {
        &self.1
    }
}
impl juniper::Context for GraphQLCtx {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, serde_json::Error>;
}

impl Handler<GraphQLData> for SledExecutor {
    type Result = Result<String, serde_json::Error>;

    fn handle(&mut self, msg: GraphQLData, ctx: &mut Self::Context) -> Self::Result {
        let logger = self.logger.clone();
        info!(self.logger, "{:#?}", msg);
        let res = msg
            .0
            .execute(&self.schema, &GraphQLCtx(self.reader.clone(), logger));
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}
