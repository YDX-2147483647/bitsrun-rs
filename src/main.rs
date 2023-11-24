mod cli;
mod client;
mod user;
mod xencode;

use anyhow::Context;
use anyhow::Result;
use cli::Args;
use cli::Commands;
use client::get_login_state;
use client::SrunClient;
use colored::Colorize;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let name = "bitsrun:".blue();

    // reusable http client
    let http_client = reqwest::Client::new();

    // commands
    match &args.command {
        // check login status
        Some(Commands::Status) => {
            let login_state = get_login_state(&http_client).await?;
            if login_state.error == "ok" {
                println!(
                    "{} {} {} {}",
                    &name.green(),
                    &login_state.online_ip.to_string().underline(),
                    format!("({})", login_state.user_name.clone().unwrap_or_default()).dimmed(),
                    "is online"
                );
            } else {
                println!(
                    "{} {} {}",
                    name,
                    login_state.online_ip.to_string().underline(),
                    "is offline"
                );
            }
            if args.verbose {
                let pretty_json = serde_json::to_string_pretty(&login_state)?;
                println!("{}", pretty_json);
            }
        }

        // login or logout
        Some(Commands::Login) | Some(Commands::Logout) => {
            let bit_user =
                user::get_bit_user(args.username.clone(), args.password.clone(), args.config)
                    .with_context(|| "unable to parse user credentials")?;

            let srun_client = SrunClient::new(
                bit_user.username,
                bit_user.password,
                Some(http_client),
                args.ip,
            )
            .await?;

            if let Some(Commands::Login) = &args.command {
                let resp = srun_client.login().await?;
                match resp.error.as_str() {
                    "ok" => println!(
                        "{} {} {} {}",
                        name.green(),
                        resp.online_ip.to_string().underline(),
                        format!("({})", resp.username.clone().unwrap_or_default()).dimmed(),
                        "logged in"
                    ),
                    _ => println!(
                        "{} failed to login, {} {}",
                        name.red(),
                        resp.error,
                        format!("({})", resp.error_msg).dimmed()
                    ),
                }

                if args.verbose {
                    let pretty_json = serde_json::to_string_pretty(&resp)?;
                    println!("{}", pretty_json);
                }
            } else if let Some(Commands::Logout) = &args.command {
                let resp = srun_client.logout().await?;
                match resp.error.as_str() {
                    "ok" => println!(
                        "{} {} {}",
                        name.green(),
                        resp.online_ip.to_string().underline(),
                        "logged out"
                    ),
                    _ => println!(
                        "{} failed to logout, {} {}",
                        name.red(),
                        resp.error,
                        format!("({})", resp.error_msg).dimmed()
                    ),
                }

                if args.verbose {
                    let pretty_json = serde_json::to_string_pretty(&resp)?;
                    println!("{}", pretty_json);
                }
            }
        }

        None => {}
    }

    Ok(())
}
