use std::{env, process, time::Duration};

use clap::Parser;
use colored::Colorize;
use crossterm::{
    cursor::{self, MoveToColumn, MoveToPreviousLine},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use futures::stream::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use unicode_segmentation::UnicodeSegmentation;

use crate::openai::Message;

mod openai;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(api_key) = env::var("OPENAI_API_KEY") else {
        println!("{} {}", "OPENAI_API_KEY not set.".red(), "Refer to step 3 here: https://help.openai.com/en/articles/5112595-best-practices-for-api-key-safety".bright_black());
        process::exit(1);
    };

    let args = Args::parse();

    let mut cmd = process::Command::new("git");
    cmd.arg("log");
    if args.short {
        cmd.arg("--oneline");
    }
    if let Some(range) = args.range {
        cmd.arg(range);
    }
    let output = match cmd.output() {
        Ok(output) => String::from_utf8(output.stdout).expect("Failed to parse output"),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let prompt_tokens = openai::count_token(&output)?;
    if prompt_tokens > args.model.context_size() {
        eprintln!(
            "Error: Git log is too long. Prompt is {} tokens, but the maximum is {}.\nTry using a smaller range or the -s flag.",
            format!("{}", prompt_tokens).purple(),
            format!("{}", args.model.context_size()).purple()
        );
        process::exit(1);
    }

    let messages = vec![
        Message::system(String::from(SYSTEM_MSG)),
        Message::user(output),
    ];

    let req = openai::Request::new(
        args.model.clone().to_string(),
        messages,
        1,
        args.temp,
        args.freq,
    );

    let json = match serde_json::to_string(&req) {
        Ok(json) => json,
        Err(e) => {
            println!("{e}");
            process::exit(1);
        }
    };

    let request_builder = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .body(json);

    let loading_ai_animation = tokio::spawn(async {
        let emoji_support =
            terminal_supports_emoji::supports_emoji(terminal_supports_emoji::Stream::Stdout);
        let frames = if emoji_support {
            vec![
                "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚",
            ]
        } else {
            vec!["/", "-", "\\", "|"]
        };
        let mut current_frame = 0;
        let mut stdout = std::io::stdout();
        loop {
            current_frame = (current_frame + 1) % frames.len();
            match execute!(
                stdout,
                Clear(ClearType::CurrentLine),
                MoveToColumn(0),
                SetForegroundColor(Color::Yellow),
                Print("Asking AI "),
                Print(frames[current_frame]),
                ResetColor
            ) {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    });

    let term_width = terminal::size()?.0 as usize;

    let mut stdout = std::io::stdout();

    let mut changelog = String::new();

    let mut es = EventSource::new(request_builder)?;
    let mut lines_to_move_up = 0;
    let mut response_tokens = 0;
    while let Some(event) = es.next().await {
        if !loading_ai_animation.is_finished() {
            loading_ai_animation.abort();
            execute!(
                std::io::stdout(),
                Clear(ClearType::CurrentLine),
                MoveToColumn(0),
            )?;
            print!("\n\n")
        }

        execute!(
            stdout,
            cursor::SavePosition,
            MoveToPreviousLine(lines_to_move_up),
        )?;
        lines_to_move_up = 0;
        match event {
            Ok(Event::Message(message)) => {
                if message.data == "[DONE]" {
                    break;
                }
                execute!(stdout, Clear(ClearType::FromCursorDown),)?;
                let resp = serde_json::from_str::<openai::Response>(&message.data)
                    .map_or_else(|_| openai::Response::default(), |r| r);
                if let Some(delta) = &resp.choices[0].delta.content {
                    changelog.push_str(delta);
                    response_tokens += 1;
                }
                let outp = format!(
                    "{}{}\n{}\n",
                    Print(format!("{}\n", "=======================").bright_black()),
                    format!(
                        "This used {} tokens costing you about {}\n",
                        format!("{}", response_tokens + prompt_tokens).purple(),
                        format!("~${:0.4}", args.model.cost(prompt_tokens, response_tokens))
                            .purple()
                    ),
                    changelog,
                );
                print!("{outp}");
                lines_to_move_up += count_lines(&outp, term_width) - 1;
            }
            Err(e) => {
                println!("{e}");
                process::exit(1);
            }
            _ => {}
        }
    }

    execute!(
        stdout,
        cursor::RestorePosition,
        Print(format!("{}\n", "=======================").bright_black()),
    )?;

    Ok(())
}

// tool to generate changelog from commit range
// ranges:
//   hash to head
//   hash to hash
//   tag to head
//   tag to tag
//   tag to hash

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///Rev range to generate changelog from
    range: Option<String>,

    ///Only use first line of commit message to reduce tokens
    #[arg(short, long)]
    short: bool,

    ///Temperature for AI
    /// 0.0 - 2.0
    #[arg(short, long, default_value = "1.0")]
    temp: f64,

    ///Frequency Penalty for AI
    /// -2.0 - 2.0
    #[arg(short, long, default_value = "0.0")]
    freq: f64,

    ///Model to use
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: openai::Model,
}

#[must_use]
pub fn count_lines(text: &str, max_width: usize) -> u16 {
    if text.is_empty() {
        return 0;
    }
    let mut line_count = 0;
    let mut current_line_width = 0;
    for cluster in UnicodeSegmentation::graphemes(text, true) {
        match cluster {
            "\r" | "\u{FEFF}" => {}
            "\n" => {
                line_count += 1;
                current_line_width = 0;
            }
            _ => {
                current_line_width += 1;
                if current_line_width > max_width {
                    line_count += 1;
                    current_line_width = cluster.chars().count();
                }
            }
        }
    }

    line_count + 1
}

const SYSTEM_MSG: &str = r#"You are now an AI that takes a range of Git commit messages as input and generates a changelog in the style of update notes using Markdown formatting. The commit messages may be in the format of a one-line summary or a multi-line description."#;
