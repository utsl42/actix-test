//! Actix web mtbl example

extern crate actix;
extern crate actix_web;

extern crate futures;
extern crate serde;
extern crate serde_cbor;
extern crate serde_json;
extern crate serde_yaml;

use actix::*;
use actix::actors::signal;
use actix_web::*;

use futures::future::Future;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
extern crate slog_term;

extern crate http;
extern crate mtbl;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate tera;

use slog::Drain;
use std::thread;
use std::sync::Arc;
use http::header;

mod mt;
mod logger;

use mt::{GetCountry, MtblExecutor};
use logger::ThreadLocalDrain;

// make git sha available in the program
include!(concat!(env!("OUT_DIR"), "/version.rs"));


/// State with MtblExecutor address
struct State {
    mt: actix::Addr<Syn, MtblExecutor>,
    logger: slog::Logger,
}

fn start_http(mt_addr: actix::Addr<Syn, MtblExecutor>, logger: slog::Logger) {
    let sys: actix::SystemRunner = actix::System::new("mtbl-example");
    let _addr = HttpServer::new(move || {
        Application::with_state(State {
            mt: mt_addr.clone(),
            logger: logger.clone(),
        }).resource("/{name}", |r| r.method(Method::GET).a(index))
    }).bind("0.0.0.0:63333")
        .unwrap()
        .start();
    sys.run();
}

lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        let mut t = compile_templates!("templates/**/*");
        t.autoescape_on(vec!["html"]);
        t
    };
}

// Async request handler
fn index(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let name = &req.match_info()["name"];
    let _guard = logger::FnGuard::new(req.state().logger.clone(),
                                      o!("name"=>name.to_owned()),
                                      "index");
    let accept_hdr = get_accept_str(req.headers().get(header::ACCEPT));

    //info!(logger, "index called");
    let movable_logger = req.state().logger.new(o!());
    req.state()
        .mt
        .send(GetCountry {
            name: name.to_owned(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(country) => match country {
                Some(c) => make_response(movable_logger, accept_hdr, c),
                None => Ok(httpcodes::HTTPNotFound.into()),
            },
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}


fn make_response(
    log: slog::Logger,
    accept: Option<String>,
    object: serde_cbor::Value,
) -> std::result::Result<actix_web::HttpResponse, actix_web::Error> {
    let _guard = logger::FnGuard::new(log, o!(), "make_response");
    let mut res = httpcodes::HttpOk.build();
    if let Some(value) = accept {
        let hstr = value.as_str();
        if hstr == "application/yaml" {
            return Ok(res.content_type("application/yaml")
                .body(serde_yaml::to_string(&object).unwrap())?);
        }
        if hstr == "application/json" {
            return res.json(&object);
        }
    }
    Ok(res.content_type("text/html")
        .body(TEMPLATES.render("country.html", &object).unwrap())?)
}

fn get_accept_str(hdr: Option<&http::header::HeaderValue>) -> Option<String> {
    match hdr {
        Some(h) => match h.to_str() {
            Ok(st) => Some(st.to_string()),
            _ => None,
        },
        None => None,
    }
}

fn main() {
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
                "sha"=>short_sha()))
        .build()
        .fuse();

    // duplicate log to both terminal and json file
    let dup_drain = slog::Duplicate::new(json_drain, term_drain);
    // make it async
    let async_drain = slog_async::Async::new(dup_drain.fuse()).build();
    // and add thread local logging
    let log = slog::Logger::root(
        ThreadLocalDrain { drain: async_drain }.fuse(), o!(),
    );

    //--- end of slog setup
    let sys = actix::System::new("mtbl-example");

    // set up MTBL lookup thread
    let mt_logger = log.new(o!("thread_name"=>"mtbl"));
    let reader = Arc::new(mtbl::Reader::open_from_path("countries.mtbl").unwrap());
    // Start mtbl executor actors
    let addr = SyncArbiter::start(3, move || MtblExecutor {
        reader: reader.clone(),
        logger: mt_logger.new(o!()),
    });

    // Start http server in its own thread
    let http_logger = log.new(o!("thread_name"=>"http"));
    thread::spawn(move || {
        start_http(addr, http_logger);
    });
    info!(log, "Started http server");

    // handle signals
    let _: actix::Addr<Syn, _> = signal::DefaultSignalsHandler::start_default();
    let _ = sys.run();
}
