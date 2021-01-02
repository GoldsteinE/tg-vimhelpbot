mod tagsdb;
mod tagsearch;
mod utils;

use std::env;
use tagsearch::TagSearcher;
use teloxide::{dispatching::Dispatcher, prelude::*, types::ParseMode};
use utils::DELETE_REGEX;

async fn handle_message(message: UpdateWithCx<Message>, tagsearcher: TagSearcher) {
    let text = message.update.text();
    let bot_reply = text
        .map(|text| tagsearcher.find_by_text(text))
        .flatten()
        .map(|answer| message.answer(answer).parse_mode(ParseMode::HTML));
    let must_delete = text
        .map(|text| DELETE_REGEX.is_match(text))
        .unwrap_or(false);

    if let Some(bot_reply) = bot_reply {
        let replied = if let Some(reply) = message.update.reply_to_message() {
            bot_reply.reply_to_message_id(reply.id).send().await.is_ok()
        } else {
            bot_reply
                .reply_to_message_id(message.update.id)
                .send()
                .await
                .is_ok()
        };
        if replied && must_delete {
            message.delete_message().send().await.ok();
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let tagsearch = TagSearcher::from_env();
    let bot = Bot::from_env();
    log::info!("Starting vim-help bot...");

    Dispatcher::new(bot)
        .messages_handler({
            |rx: DispatcherHandlerRx<Message>| async move {
                rx.for_each(|message| handle_message(message, tagsearch.clone()))
                    .await
            }
        })
        .dispatch()
        .await;
}
