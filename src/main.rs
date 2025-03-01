mod commands;
mod context;
mod discord;
mod irc;
mod shazam;
mod utils;

use crate::context::Context;
use crate::discord::CommandContext;
use crate::irc::IrcClientExt;
use discord::get_serenity_client;
use dotenvy::dotenv;
use serenity::all::ChannelId;
use std::env;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let mut discord_client = get_serenity_client().await;
    let mut irc_client = irc::get_irc_client().await;
    let discord_http = discord_client.http.clone();
    let discord_cache = discord_client.cache.clone();
    let irc_sender = irc_client.sender();
    let discord_channel: ChannelId = env::var("DISCORD_CHANNEL_ID")
        .expect("DISCORD_CHANNEL_ID must be set")
        .parse()
        .expect("DISCORD_CHANNEL_ID must be a number");
    let irc_channel = env::var("IRC_MAIN_CHANNEL").expect("IRC_MAIN_CHANNEL must be set");
    let command_prefix = env::var("COMMAND_PREFIX").expect("COMMAND_PREFIX must be set");
    let discord_webhook_url =
        env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL must be set");
    let shazam_discord_channel: ChannelId = env::var("SHAZAM_DISCORD_CHANNEL_ID")
        .expect("SHAZAM_DISCORD_CHANNEL_ID must be set")
        .parse()
        .expect("SHAZAM_DISCORD_CHANNEL_ID must be a number");
    let shazam_irc_channel =
        env::var("SHAZAM_IRC_CHANNEL").expect("SHAZAM_IRC_CHANNEL must be set");

    let context = Context {
        discord_http,
        discord_cache,
        discord_channel,
        discord_webhook_url,
        irc_sender,
        irc_channel,
        command_prefix,
        last_track: Arc::new(Mutex::new(None)),
        shazam_discord_channel,
        shazam_irc_channel,
    };

    discord_client
        .data
        .write()
        .await
        .insert::<CommandContext>(context.clone());

    let discord_handle = tokio::spawn(async move { discord_client.start().await });
    let irc_context = context.clone();
    let irc_handle = tokio::spawn(async move { irc_client.start(irc_context).await });
    let shazam_context = context.clone();
    let shazam_handle = tokio::spawn(async move { shazam::start(shazam_context).await });
    let now_playing_handle = tokio::spawn(async move { utils::now_playing_loop(context).await });

    _ = tokio::join!(
        discord_handle,
        irc_handle,
        shazam_handle,
        now_playing_handle
    );
}
