mod commands;
mod jobs;

use crate::Result;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use clap::Parser;
use serenity::all::{
    CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, Verifier,
};
use serenity::http::Http;
use std::sync::Arc;

pub(super) struct DiscordBotState {
    verifier: Verifier,
    http_client: Http,
}

#[derive(Parser)]
pub(crate) struct DiscordBotConfig {
    #[arg(long, value_parser = parse_str_to_hex, env = "GENNY_PUBLIC_KEY")]
    public_key: [u8; 32],
    #[arg(long, env = "GENNY_BOT_TOKEN")]
    bot_token: String,
}

impl DiscordBotState {
    pub(super) fn configure(config: DiscordBotConfig) -> Result<Self> {
        let verifier = Verifier::try_new(config.public_key)?;
        let http_client = Http::new(&config.bot_token);

        let state = Self {
            verifier,
            http_client,
        };

        Ok(state)
    }
}

fn parse_str_to_hex(str: &str) -> Result<[u8; 32]> {
    let bytes: [u8; 32] = const_hex::decode_to_array(&str)?;

    Ok(bytes)
}

#[tracing::instrument(name = "Interaction", level = "trace", skip_all)]
pub(super) async fn handle_interaction(
    State(verifier): State<Arc<Verifier>>,
    headers: HeaderMap,
    body: Bytes,
) -> std::result::Result<Json<CreateInteractionResponse>, StatusCode> {
    let signature_header = headers
        .get("X-Signature-Ed25519")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let signature = signature_header
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let timestamp_header = headers
        .get("X-Signature-Timestamp")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let timestamp = timestamp_header
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    verifier
        .verify(signature, timestamp, &body)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let Json(interaction): Json<Interaction> =
        Json::from_bytes(&body).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let response = match interaction {
        Interaction::Ping(c) => CreateInteractionResponse::Pong,
        Interaction::Command(c) => {
            todo!()
        }
        Interaction::Autocomplete(a) => {
            todo!()
        }
        Interaction::Component(com) => {
            todo!()
        }
        Interaction::Modal(fz) => {
            todo!()
        }
        _ => todo!(),
    };

    Ok(response.into())
}
