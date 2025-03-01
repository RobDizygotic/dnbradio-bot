use crate::{commands, context};
use log::error;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

pub struct CommandContext;

impl TypeMapKey for CommandContext {
    type Value = context::Context;
}

pub async fn get_serenity_client() -> Client {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_PRESENCES;

    Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client")
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut context = ctx
            .data
            .read()
            .await
            .get::<CommandContext>()
            .unwrap()
            .clone();

        if msg.channel_id != context.discord_channel || msg.webhook_id.is_some() || msg.author.bot {
            return;
        }

        context
            .send_to_irc(
                format!(
                    "<{}> {}",
                    msg.author_nick(&context.discord_http)
                        .await
                        .unwrap_or(msg.author.global_name.unwrap_or(msg.author.name)),
                    msg.content
                )
                .as_str(),
            )
            .await;

        if msg.content.starts_with(&context.command_prefix) {
            let command = msg.content[1..].to_owned();
            if let Err(error) = commands::handle_command(&mut context, &command, false).await {
                error!("Error handling command {}: {:?}", command, error);
            }
        }
    }
}
