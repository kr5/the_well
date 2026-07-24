//! Renders channel-agnostic `channel_core::RenderableNode` data as
//! spoken-prompt text for Exotel's TTS, and maps DTMF digit input back to
//! a decision-tree option — the "TTS rendering... in at least Hindi +
//! English" and "DTMF... answer capture" halves of E8.F3.T2/T3. No
//! decision logic lives here, per docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.7.
//!
//! A phone call has no visual keypad legend, so every option must be
//! spoken with its digit explicitly ("for X, press 1; for Y, press 2")
//! rather than shown — this is the IVR-specific rendering job Telegram's
//! inline keyboard and WhatsApp's interactive list don't need.

use channel_core::RenderableQuestion;

/// Single DTMF digits 1-9 map to options by position; 0 is reserved
/// (commonly used by IVR systems for "repeat"/"operator" in a fuller
/// implementation, not assigned to any option here).
pub const MAX_DTMF_OPTIONS: usize = 9;

/// Builds the full spoken prompt for a question: the prompt/help text,
/// followed by one "for X, press N" line per option. Truncated at
/// `MAX_DTMF_OPTIONS` with a logged warning if a question offers more
/// options than single DTMF digits can address — mirroring
/// `bot_whatsapp::render_whatsapp`'s handling of its own platform-imposed
/// option-count ceiling. A question needing multi-digit DTMF sequences is
/// a real future extension (E8.F3's "full implementation"), not something
/// this scaffold silently mishandles today.
pub fn render_spoken_prompt(question: &RenderableQuestion) -> String {
    let mut spoken = question.prompt.clone();
    if let Some(help) = &question.help_text {
        spoken.push_str(". ");
        spoken.push_str(help);
    }

    if question.options.len() > MAX_DTMF_OPTIONS {
        tracing::warn!(
            node_id = %question.node_id,
            option_count = question.options.len(),
            "question has more options than single-digit DTMF (9) supports; trailing options are unreachable on this channel"
        );
    }

    for (index, option) in question.options.iter().take(MAX_DTMF_OPTIONS).enumerate() {
        spoken.push_str(". For ");
        spoken.push_str(&option.label);
        spoken.push_str(", press ");
        spoken.push_str(&(index + 1).to_string());
    }

    spoken
}

/// Maps a single DTMF digit string ("1".."9") to the corresponding
/// option's `value`, by position — the standard "press 1 for X" IVR
/// convention. Returns `None` for anything else (multi-digit input, "0",
/// non-numeric junk, or a digit beyond the number of options actually
/// offered), leaving the caller to decide how to reprompt.
pub fn map_digit_to_option<'a>(question: &'a RenderableQuestion, digits: &str) -> Option<&'a str> {
    let digit: usize = digits.trim().parse().ok()?;
    if digit == 0 {
        return None;
    }
    question.options.get(digit - 1).map(|option| option.value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use channel_core::RenderableOption;

    fn sample_question() -> RenderableQuestion {
        RenderableQuestion {
            node_id: "n1".into(),
            prompt: "Are you a first-time voter?".into(),
            help_text: None,
            options: vec![
                RenderableOption { value: "yes".into(), label: "Yes".into() },
                RenderableOption { value: "no".into(), label: "No".into() },
            ],
        }
    }

    #[test]
    fn render_spoken_prompt_mentions_every_option_and_its_digit() {
        let spoken = render_spoken_prompt(&sample_question());
        assert!(spoken.contains("press 1"));
        assert!(spoken.contains("press 2"));
        assert!(spoken.contains("Yes"));
        assert!(spoken.contains("No"));
    }

    #[test]
    fn map_digit_to_option_resolves_by_position() {
        let question = sample_question();
        assert_eq!(map_digit_to_option(&question, "1"), Some("yes"));
        assert_eq!(map_digit_to_option(&question, "2"), Some("no"));
    }

    #[test]
    fn map_digit_to_option_rejects_out_of_range_and_non_numeric_input() {
        let question = sample_question();
        assert_eq!(map_digit_to_option(&question, "0"), None);
        assert_eq!(map_digit_to_option(&question, "9"), None);
        assert_eq!(map_digit_to_option(&question, "abc"), None);
    }
}
