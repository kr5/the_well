use std::sync::Arc;

use channel_core::{render_node, RenderableNode, SessionStore};
use core_domain::{answer, create_session, get_current_node, DecisionTree, EngineState};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

use crate::commands::Command;
use crate::locale::resolve_locale;
use crate::render_telegram::{render_question, render_terminal, NOT_OFFICIAL_BANNER};
use crate::state::{HandlerResult, State, TelegramDialogue};

/// Handles every plain message: a bot command if the text parses as one,
/// silently ignored otherwise (a citizen mid-walkthrough who types free
/// text instead of tapping a button gets no reply here — the inline
/// keyboard is the only input surface for answering a question, by
/// design, since free text was never a valid decision-tree option and
/// silently accepting it would risk misinterpreting an off-tree answer).
pub async fn handle_command(
    bot: Bot,
    dialogue: TelegramDialogue,
    msg: Message,
    me: Me,
    tree: Arc<DecisionTree>,
    sessions: SessionStore,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(());
    };
    let Ok(command) = Command::parse(text, me.username()) else {
        return Ok(());
    };

    match command {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::About => {
            bot.send_message(msg.chat.id, NOT_OFFICIAL_BANNER).await?;
        }
        Command::Start => {
            let locale = resolve_locale(msg.from.as_ref());
            let engine_state = create_session(&tree)?;
            let token = sessions.create(engine_state.clone()).await;
            dialogue.update(State::Walkthrough { session_token: token }).await?;
            send_current_node(&bot, msg.chat.id, &tree, &engine_state, locale).await?;
        }
    }

    Ok(())
}

/// A button press arriving while the dialogue has no active walkthrough
/// (session expired, bot restarted, or a stale keyboard from an old chat)
/// — tell the citizen to restart rather than silently doing nothing.
pub async fn handle_stray_callback(bot: Bot, q: CallbackQuery) -> HandlerResult {
    bot.answer_callback_query(q.id).await?;
    if let Some(message) = q.regular_message() {
        bot.send_message(message.chat.id, "Send /start to begin.").await?;
    }
    Ok(())
}

pub async fn handle_answer_callback(
    bot: Bot,
    dialogue: TelegramDialogue,
    session_token: String,
    q: CallbackQuery,
    tree: Arc<DecisionTree>,
    sessions: SessionStore,
) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(message) = q.regular_message() else {
        return Ok(());
    };
    let Some(value) = q.data.as_ref() else {
        return Ok(());
    };

    let locale = resolve_locale(Some(&q.from));

    let Some(current_state) = sessions.get(&session_token).await else {
        bot.send_message(
            message.chat.id,
            "Your session expired. Send /start to begin again.",
        )
        .await?;
        dialogue.update(State::Idle).await?;
        return Ok(());
    };

    let next_state: EngineState = match answer(&tree, &current_state, value) {
        Ok(next) => next,
        Err(err) => {
            tracing::warn!(error = %err, node_id = %current_state.current_node_id, "invalid answer submitted via Telegram callback");
            bot.send_message(
                message.chat.id,
                "That option is no longer valid — send /start to begin again.",
            )
            .await?;
            sessions.remove(&session_token).await;
            dialogue.update(State::Idle).await?;
            return Ok(());
        }
    };

    let node = get_current_node(&tree, &next_state)?;

    if node.is_terminal() {
        let RenderableNode::Terminal(terminal) = render_node(node, locale) else {
            unreachable!("node.is_terminal() just confirmed this node renders as a terminal");
        };
        bot.send_message(message.chat.id, render_terminal(&terminal)).await?;
        sessions.remove(&session_token).await;
        dialogue.update(State::Idle).await?;
    } else {
        sessions.put(session_token, next_state.clone()).await;
        send_current_node(&bot, message.chat.id, &tree, &next_state, locale).await?;
    }

    Ok(())
}

async fn send_current_node(
    bot: &Bot,
    chat_id: ChatId,
    tree: &DecisionTree,
    state: &EngineState,
    locale: &str,
) -> HandlerResult {
    let node = get_current_node(tree, state)?;
    match render_node(node, locale) {
        RenderableNode::Question(question) => {
            let (text, keyboard) = render_question(&question);
            bot.send_message(chat_id, text).reply_markup(keyboard).await?;
        }
        RenderableNode::Terminal(terminal) => {
            bot.send_message(chat_id, render_terminal(&terminal)).await?;
        }
    }
    Ok(())
}
