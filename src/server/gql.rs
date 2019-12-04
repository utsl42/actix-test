use juniper;
use juniper::http::GraphQLRequest;
use juniper::GraphQLObject;
use serde_cbor;
use serde_derive::{Deserialize, Serialize};
use slog::error;
use std;
use std::fs::File;
use std::io;

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

pub struct GraphQLCtx(pub std::sync::Arc<sled::Tree>, pub slog::Logger);
impl GraphQLCtx {
    pub fn get(&self, name: String) -> std::option::Option<std::vec::Vec<u8>> {
        self.0.get(name).ok()?.and_then(|val| Some(val.to_vec()))
    }

    pub fn iter(&self) -> sled::Iter {
        self.0.iter()
    }

    fn logger(&self) -> &slog::Logger {
        &self.1
    }
}
impl juniper::Context for GraphQLCtx {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(pub GraphQLRequest);

#[derive(Serialize)]
struct DataResult<'a> {
    data: &'a juniper::Value,
}

pub fn dump_schema(
    s: &Schema,
    tree: std::sync::Arc<sled::Tree>,
    logger: slog::Logger,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(res) = juniper::introspect(
        s,
        &GraphQLCtx(tree, logger),
        juniper::IntrospectionFormat::All,
    ) {
        let bw = io::BufWriter::new(File::create("frontend/graphql_schema.json")?);
        serde_json::to_writer_pretty(bw, &DataResult { data: &res.0 })?;
    }
    Ok(())
}
