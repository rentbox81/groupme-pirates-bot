pub mod config;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::Layer;
pub mod auth;
pub mod error;
pub mod models;
pub mod google_client;
pub mod groupme_client;
pub mod weather_client;
pub mod service;
pub mod parser;
pub mod conversational_parser;
pub mod reminder;
pub mod conversation_context;
pub mod moderators;
pub mod team_facts;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use tracing::{info, error, warn};
use tracing_actix_web::TracingLogger;
use std::sync::Arc;

use crate::config::Config;
use crate::service::BotService;
use crate::parser::CommandParser;
use crate::models::GroupMeMessage;
use crate::reminder::ReminderScheduler;

// Application state
struct AppState {
    bot_service: BotService,
    command_parser: CommandParser,
    moderators_store: moderators::ModeratorsStore,
    config: config::Config,
}

#[post("/webhook")]
async fn webhook(req_body: String, data: web::Data<AppState>) -> impl Responder {
    // Debug: Log raw webhook payload to see what GroupMe sends
    info!("Raw GroupMe webhook payload: {}", req_body);
    let msg: GroupMeMessage = match serde_json::from_str(&req_body) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to parse GroupMe message: {}", e);
            return HttpResponse::BadRequest().body("Invalid JSON");
        }
    };

    // Ignore messages from the bot itself
    if msg.sender_type == "bot" {
        return HttpResponse::Ok().body("OK");
    }

    info!("Received message from {}: '{}'", msg.name, msg.text);

    // Parse the command
    let command = match data.command_parser.parse_message(&msg.text, Some(&msg.name), Some(&msg.user_id), &msg.attachments).await {
        Ok(Some(cmd)) => cmd,
        Ok(None) => {
            // Message not directed at bot, ignore
            return HttpResponse::Ok().body("OK");
        }
        Err(e) => {
            // Conversational error with friendly message
            warn!("Conversational parsing resulted in friendly error: {}", e);
            let error_response = format!("{}", e);
            if let Err(send_error) = data.bot_service.send_response(&error_response).await {
                error!("Failed to send friendly response: {}", send_error);
            }
            return HttpResponse::Ok().body("OK");
        }
    };

    // Handle the command
    match data.bot_service.handle_command(command, Some(&msg.name), Some(&msg.user_id), &data.moderators_store).await {
        Ok(response) => {
            if let Err(e) = data.bot_service.send_response(&response).await {
                error!("Failed to send response: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to handle command: {}", e);
            // Send a friendly error instead of technical error codes
            let error_response = "🏴‍☠️ Ahoy! I ran into a problem with that request. Try again in a moment, matey! ⚾";
            if let Err(send_error) = data.bot_service.send_response(error_response).await {
                error!("Failed to send error response: {}", send_error);
            }
        }
    }

    HttpResponse::Ok().body("OK")
}

#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "GroupMe Bot",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "groupme-bot.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR);
    
    let console_layer = tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());
    
    let _guard = guard; // Keep guard alive for the lifetime of the program
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting GroupMe bot '{}' on port {}", config.groupme_bot_name, config.port);

    // Start reminder scheduler
    let reminder_scheduler = Arc::new(ReminderScheduler::new(config.clone()));
    reminder_scheduler.start();
    info!("Reminder scheduler initialized");

    // Create services
    let bot_service = BotService::new(config.clone());
    let command_parser = CommandParser::new(config.groupme_bot_name.clone());

    // Create application state
    let app_state = web::Data::new(AppState {
        bot_service,
        command_parser,
        moderators_store: moderators::ModeratorsStore::new(),
        config: config.clone(),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(TracingLogger::default())
            .service(webhook)
            .service(health_check)
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}
