//! Actix web mtbl example

use actix::prelude::*;
use actix_web::{http, middleware::cors::Cors, App, Error, HttpRequest, HttpResponse};
use actix_files as fs;
use futures;
use futures::future::Future;
use juniper;
use sled;
use slog;
use slog::Drain;
use slog::{info, o};
use slog_async;
use slog_term;
use std::fs::File;
use std::io;

mod gql;
mod logger;
mod mt;

use crate::logger::ThreadLocalDrain;
use crate::mt::SledExecutor;

// make git sha available in the program
include!(concat!(env!("OUT_DIR"), "/version.rs"));

/// State with SledExecutor address
struct State {
    mt: actix::Addr<SledExecutor>,
    logger: slog::Logger,
}

fn start_http(mt_addr: actix::Addr<SledExecutor>, logger: slog::Logger) {
    actix_web::HttpServer::new(move || {
        let cors = Cors::new()
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .data(State {
                mt: mt_addr.clone(),
                logger: logger.clone(),
            })
            .wrap(cors)
            .route("/graphql", actix_web::web::post().to_async(graphql))
            .route("/graphiql", actix_web::web::get().to(graphiql))
            .route("/playground", actix_web::web::get().to(playground))
            .service(
                fs::Files::new("/", "./frontend/dist/")
                    .index_file("index.html"),
            )
    })
    .bind("0.0.0.0:63333")
    .unwrap()
    .start();
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

fn graphql((st, data): (actix_web::web::Data<State>, actix_web::web::Json<gql::GraphQLData>)) -> impl Future<Item=HttpResponse, Error=Error> {
    st.mt
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn build_db(db: std::sync::Arc<sled::Tree>) -> Result<()> {
    let br = io::BufReader::new(File::open("countries.json")?);
    let data: serde_json::Value = serde_json::from_reader(br)?;

    if data.is_array() {
        let decoded: &Vec<serde_json::Value> = data.as_array().unwrap();
        for object in decoded.iter() {
            if let Some(&serde_json::Value::String(ref name)) = object.pointer("/cca3") {
                db.set(name, serde_cbor::to_vec(object)?)?;
            }
        }
    }
    db.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    //--- set up slog

    // set up terminal logging
    let decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::CompactFormat::new(decorator).build().fuse();

    // json log file
    let logfile = std::fs::File::create("/tmp/actix-test.log").unwrap();
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
    let tree = sled::Db::start_default("countries_db")?.open_tree(b"countries".to_vec())?;

    // dump the graphql schema, which needs the database because of the graphql context
    gql::dump_schema(&gql::create_schema(), tree.clone(), log.clone())?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
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

    actix::System::run(move || {
        // set up MTBL lookup thread
        let mt_logger = log.new(o!("thread_name"=>"sled"));

        // Start sled executor actors
        let addr = SyncArbiter::start(3, move || {
            SledExecutor::new(tree.clone(), mt_logger.new(o!()))
        });

        // Start http server in its own thread
        let http_logger = log.new(o!("thread_name"=>"http"));
        start_http(addr, http_logger);
        info!(log, "Started http server");

        // handle signals
//        let _ = signal::DefaultSignalsHandler::start_default();
    });
    Ok(())
}
