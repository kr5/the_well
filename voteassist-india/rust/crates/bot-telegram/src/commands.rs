use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "VoteAssist India commands:")]
pub enum Command {
    #[command(description = "show this help text")]
    Help,
    #[command(description = "start (or restart) the voter-guidance walkthrough")]
    Start,
    #[command(description = "read the non-official disclaimer")]
    About,
}
