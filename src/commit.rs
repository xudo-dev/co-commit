use crate::args::CommandArgs;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use spinners::{Spinner, Spinners};

pub async fn generate_commit(args: &CommandArgs) -> Result<(), Box<dyn std::error::Error>> {
    let api_token = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENAI_API_KEY environment variable not set");
        std::process::exit(1);
    });

    println!("Checking for staged files...");

    let git_staged_cmd = std::process::Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .output()
        .expect("Failed to execute git diff --cached --name-only");

    let git_staged = String::from_utf8(git_staged_cmd.stdout).unwrap();

    if git_staged.is_empty() {
        eprintln!("No staged files found. Aborting.");
        std::process::exit(1);
    }

    let is_repo = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("Failed to execute git rev-parse --is-inside-work-tree");

    println!("Checking if current directory is a git repository...");

    if !is_repo.stdout.starts_with(b"true") {
        eprintln!("Current directory is not a git repository. Aborting.");
        std::process::exit(1);
    }

    let client = openai_api::Client::new(&api_token);
    let output = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .output()
        .expect("Couldn't find diff.")
        .stdout;
    let output = String::from_utf8(output.clone()).unwrap();

    println!("Generating diff...{:?}", output);

    if !args.dry_run {
        println!("Generating commit message...");
    }

    let ai_prompt = openai_api::api::CompletionArgs::builder()
    .prompt(format!(
        "git diff HEAD\\^!\n{:#?}\n\n# Write a commit message describing the changes and the reasoning behind them\ngit commit -F- <<EOF",
        output
    ))
    .engine(openai_api::api::Engine::Davinci)
    .temperature(0.0)
    .max_tokens(2000)
    .stop(vec!["EOF".into()]).clone();

    println!("prompt generated {:?}", ai_prompt.build());
    return Ok(());

    let mut spinner = Spinner::new(Spinners::Dots12, "Generating commit message...".into());
    let completion = client.complete_prompt_sync(ai_prompt.build().unwrap()).expect("Failed to generate commit message.");

    println!("Commit message generated!");
    spinner.stop();

    let commit_message = completion.choices[0].text.clone();

    if args.dry_run {
        println!("{}", commit_message);
        return Ok(());
    } else {
        println!(
            "Proposed Commit:\n------------------------------\n{}\n------------------------------",
            commit_message
        );


        if !args.force {
            let mut confirm = String::new();
            print!("Commit? [y/N] ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut confirm).unwrap();

            if !confirm.starts_with("y") {
                println!("Commit aborted");
                return Ok(());
            }
        }

        let mut commit = Command::new("git");
        commit.arg("commit").arg("-F-").stdin(Stdio::piped());

        if args.review {
            commit.arg("--edit");
        }

        if args.force {
            commit.arg("--no-verify");
        }

        let mut commit = commit.spawn().expect("Failed to execute git commit -F-");

        commit
            .stdin
            .as_mut()
            .unwrap()
            .write_all(commit_message.as_bytes())
            .unwrap();

        commit.wait().unwrap();
        Ok(())
    }

}