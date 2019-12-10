//! Actix web mtbl example

use actix_web::{http, App, Error, HttpRequest, HttpResponse};
use actix_cors::Cors;
use actix_files as fs;
use juniper;
use sled;
use slog;
use slog::Drain;
use slog::{info, o};
use slog_async;
use slog_term;
use std::fs::File;
use std::io;
use std::sync::Arc;
use futures::future::Future;
use futures::future::TryFutureExt;

mod gql;
mod logger;
mod mt;

use crate::logger::ThreadLocalDrain;
use crate::mt::SledExecutor;
use actix_web::error::ErrorBadGateway;

// make git sha available in the program
include!(concat!(env!("OUT_DIR"), "/version.rs"));

/// State with SledExecutor address
struct State {
    mt: Arc<SledExecutor>,
    logger: slog::Logger,
}


fn graphiql(req: HttpRequest) -> HttpResponse {
    let html = juniper::graphiql::graphiql_source("http://localhost:63333/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn playground(req: HttpRequest) -> HttpResponse {
    let html = juniper::http::playground::playground_source("http://localhost:63333/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql((st, data): (actix_web::web::Data<State>, actix_web::web::Json<gql::GraphQLData>))
           -> std::result::Result<String, actix_web::Error> {
    // This doesn't really seem right, but it apparently converts the Future that map_err returns back to
    // a Result, which the async fn turns back into a Future. Hoping rustc can optimize this away...
    actix_web::web::block(move || st.mt.handle(data.0))
        .map_err(|e| ErrorBadGateway(e)
        ).await
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn build_db(db: std::sync::Arc<sled::Tree>) -> Result<()> {
    let br = io::BufReader::new(File::open("/countries.json")?);
    let data: serde_json::Value = serde_json::from_reader(br)?;

    if data.is_array() {
        let decoded: &Vec<serde_json::Value> = data.as_array().unwrap();
        for object in decoded.iter() {
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/cca3") {
                db.insert(name, serde_cbor::to_vec(object)?)?;
            }
        }
    }
    db.flush()?;
    Ok(())
}

#[actix_rt::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //--- set up slog

    // set up terminal logging
    let decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::CompactFormat::new(decorator).build().fuse();

    // json log file
    let logfile = std::fs::File::create("/var/tmp/actix-test.log").unwrap();
    let json_drain = slog_json::Json::new(logfile)
        .add_default_keys()
        // include source code location
        .add_key_value(o!("place" =>
           slog::FnValue(move |info| {
               format!("{}::({}:{})",
                       info.module(),
                       info.file(),
                       info.line(),
                )}),
                "sha"=>VERGEN_SHA_SHORT))
        .build()
        .fuse();

    // duplicate log to both terminal and json file
    let dup_drain = slog::Duplicate::new(json_drain, term_drain);
    // make it async
    let async_drain = slog_async::Async::new(dup_drain.fuse()).build();
    // and add thread local logging
    let log = slog::Logger::root(ThreadLocalDrain { drain: async_drain }.fuse(), o!());
    //--- end of slog setup

    //--- set up sled database
    let tree = Arc::from(sled::Db::open("/var/tmp/countries_db")?.open_tree(b"countries".to_vec())?);

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // dump the graphql schema, which needs the database because of the graphql context
        gql::dump_schema(&gql::create_schema(), tree.clone(), log.clone())?;
        return Ok(());
    }

    let build_tree = tree.clone();
    // check to see if there's a specific key, and if it's missing, assume the tree is empty, and load data
    match build_tree.get(b"DEU")? {
        Some(_) => info!(log, "tree already initialized"),
        None => {
            info!(log, "building tree");
            build_db(build_tree)?;
        }
    }
    // --- end of database setup

    let mt_logger = log.new(o!("thread_name"=>"sled"));
    let http_logger = log.new(o!("thread_name"=>"http"));

    actix_web::HttpServer::new(move || {
        let cors = Cors::new()
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .finish();

        App::new()
            .data(State {
                mt: Arc::from(SledExecutor::new(tree.clone(), mt_logger.new(o!()))),
                logger: http_logger.clone(),
            })
            .wrap(cors)
            .route("/graphql", actix_web::web::post().to(graphql))
            .route("/graphiql", actix_web::web::get().to(graphiql))
            .route("/playground", actix_web::web::get().to(playground))
            .service(
                fs::Files::new("/", "./frontend/dist/")
                    .index_file("index.html"),
            )
    })
        .bind("0.0.0.0:63333")?
        .workers(1)
        .start()
        .await?;
    info!(log, "Started http server");
    Ok(())
}
