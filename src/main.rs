use actix_files::NamedFile;
use actix_web::{
    error, guard,
    http::{self, KeepAlive},
    web, App, HttpResponse, HttpServer, Responder,
};
use askama::Template;

use log::{debug, error, info, warn};
use mongodb::Client;

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use template_variables::IndexTemplate;

use std::{fs::File, process::exit};

use crate::template_variables::NotFoundTemplate;

mod template_variables;

const HOST_IP: &str = "0.0.0.0"; // Local connection
const PORT: u16 = 8100;

// pub const MONGODB_PATH: &str = "mongodb://127.0.0.1/"; // Local connection
pub const MONGODB_PATH: &str = "mongodb://10.20.30.20:27017/";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // This is a macro that allows for multiple loggers to be used at once
    match CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Stdout,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("budget_rs.log")?,
        ),
    ]) {
        Ok(()) => debug!("Logger initialized."),
        Err(e) => {
            error!("Error initializing logger: {e:?}");
            exit(1);
        }
    }

    let client = Client::with_uri_str(MONGODB_PATH).await;

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(client.clone()))
            .service(
                actix_files::Files::new("/static", "./static")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(
                web::resource("/favicon")
                    .guard(guard::Method(http::Method::GET))
                    .to(send_favicon_icon),
            )
            .service(
                web::resource("/")
                    .guard(guard::Method(http::Method::GET))
                    .to(index),
            )
            .service(
                web::resource("/index")
                    .guard(guard::Method(http::Method::GET))
                    .to(send_css_file),
            )
            .service(
                web::resource("/index.css.map")
                    .guard(guard::Method(http::Method::GET))
                    .to(css_map_file),
            )
            .default_service(
                // Takes every not found to the 404 page and 404 response code
                web::route()
                    .guard(
                        guard::Any(guard::Patch())
                            .or(guard::Put())
                            .or(guard::Delete())
                            .or(guard::Get())
                            .or(guard::Post())
                            .or(guard::Head())
                            .or(guard::Options()),
                    )
                    .to(not_found),
            )
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .bind((HOST_IP, PORT))
    .unwrap_or_else(|_| {
        warn!("Error binding to port {}.", PORT);
        std::process::exit(1); // This is expected behavior if the port is already in use
    })
    // .disable_signals() // Disable the signals to allow the OS to handle the signals
    .shutdown_timeout(3)
    .workers(2)
    .run()
    .await
}

pub async fn index() -> HttpResponse {
    let index_template = IndexTemplate {
        title: "Index",
        current_amount: 1512.48,
        total_expenses: 5836.28,
        total_income: 7348.76,
    };

    let response_body = index_template.render().unwrap_or_else(|_| {
        error!("Error rendering index template.");
        String::from("Error rendering index template.")
    });

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("X-Frame-Options", "DENY"))
        .body(response_body)
}

pub async fn not_found() -> impl Responder {
    warn!("404 Not Found");
    let not_found_template = NotFoundTemplate {
        title: "404",
        index: "404",
    };

    let response_body = not_found_template.render().unwrap_or_else(|_| {
        error!("Error rendering 404 Not Found template.");
        String::from("Error rendering 404 Not Found template.")
    });

    info!("404 Not Found template rendered");

    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(response_body)
}

pub async fn send_favicon_icon() -> NamedFile {
    NamedFile::open("static/images/favicon.png")
        .map_err(|_| error::ErrorNotFound("File not found"))
        .unwrap_or_else(|_| {
            error!("Error sending favicon icon.");
            std::process::exit(1);
        })
}

pub async fn send_css_file() -> impl Responder {
    let css_file_data = include_str!("../static/css/index.css");

    HttpResponse::Ok()
        .content_type("text/css")
        .body(css_file_data)
}

pub async fn css_map_file() -> impl Responder {
    let css_map_file_data = include_str!("../static/css/index.css.map");

    HttpResponse::Ok()
        .content_type("application/json")
        .body(css_map_file_data)
}
