//! Actix web mtbl example

extern crate actix;
extern crate actix_web;

extern crate futures;
extern crate serde;
extern crate serde_cbor;
extern crate serde_json;
extern crate serde_yaml;

extern crate thread_id;
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

use mt::{GetCountry, MtblExecutor};

// make git sha available in the program
include!(concat!(env!("OUT_DIR"), "/version.rs"));

thread_local!(static TL_THREAD_ID: usize = thread_id::get());

/// State with MtblExecutor address
struct State {
    mt: actix::Addr<Syn, MtblExecutor>,
    logger: slog::Logger,
}

lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        let mut t = compile_templates!("templates/**/*");
        t.autoescape_on(vec!["html"]);
        t
    };
}

fn make_response(
    accept: Option<String>,
    object: serde_cbor::Value,
) -> std::result::Result<actix_web::HttpResponse, actix_web::Error> {
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

/// Async request handler
fn index(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let name = &req.match_info()["name"];
    let logger = req.state().logger.new(o!("name"=>name.to_owned()));
    let accept_hdr = get_accept_str(req.headers().get(header::ACCEPT));

    info!(logger, "index called");
    req.state()
        .mt
        .send(GetCountry {
            name: name.to_owned(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(country) => match country {
                Some(c) => make_response(accept_hdr, c),
                None => Ok(httpcodes::HTTPNotFound.into()),
            },
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}

fn start_http(mt_addr: actix::Addr<Syn, MtblExecutor>, logger: slog::Logger) {
    let sys = actix::System::new("mtbl-example");
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

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let tdrain = slog_term::FullFormat::new(decorator).build().fuse();

    let logfile = std::fs::File::create("/tmp/actix-test.log").unwrap();
    let sdrain = slog_json::Json::new(logfile)
        .add_default_keys()
        .add_key_value(o!("place" =>
           slog::FnValue(move |info| {
               format!("{}::({}:{})",
                       info.module(),
                       info.file(),
                       info.line(),
                )})))
        .build()
        .fuse();

    let log = slog::Logger::root(
        std::sync::Mutex::new(slog::Duplicate::new(sdrain, tdrain)).fuse(),
        o!("version" => "0.1.0"),
    );

    let logger = log.new(o!("host"=>"localhost",
        "port"=>8080,
        "thread"=>slog::FnValue(|_| {
            TL_THREAD_ID.with(|id| { *id })
        }),
        "sha"=>short_sha(),
        ));

    let sys = actix::System::new("mtbl-example");
    let _: actix::Addr<Syn, _> = signal::DefaultSignalsHandler::start_default();

    let mt_logger = logger.new(o!("thread_name"=>"mtbl"));
    let reader = Arc::new(mtbl::Reader::open_from_path("countries.mtbl").unwrap());
    // Start mtbl executor actors
    let addr = SyncArbiter::start(3, move || MtblExecutor {
        reader: reader.clone(),
        logger: mt_logger.new(o!()),
    });

    let http_logger = logger.new(o!("thead_name"=>"http"));
    // Start http server
    thread::spawn(move || {
        start_http(addr, http_logger);
    });
    info!(logger, "Started http server");
    let _ = sys.run();
}
