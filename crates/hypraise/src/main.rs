use clap::{Parser, Subcommand};
use hypraise::desktop::{AppInfo, AppQuery, ExecCommand};
use hypraise::wm::{self, ShellCommand, WindowClass};
use std::io::Write;
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/halo.sock";

#[derive(Parser, Debug)]
#[command(name = "hypraise", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The application name or window class (used to find desktop entry)
    name: Option<String>,

    /// Explicitly specify the window class to match (overrides desktop entry and name)
    #[arg(short = 'c', long)]
    class: Option<String>,

    /// The command to execute if the window is not found (overrides desktop entry)
    #[arg(short = 'e', long)]
    exec: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Show the Halo menu.
    Show,
    /// Hide the Halo menu
    Hide,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Show) => send_command("show"),
        Some(Commands::Hide) => send_command("hide"),
        None => {
            if let Some(query) = cli.name {
                run_or_raise(query, cli.class, cli.exec)
            } else {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                Ok(())
            }
        }
    }
}

fn run_or_raise(query: String, class: Option<String>, exec: Option<String>) -> anyhow::Result<()> {
    let app = AppInfo::new(
        &AppQuery::from(query.clone()),
        class.map(WindowClass::new),
        exec.map(ExecCommand::from),
    );

    if app.exec.is_empty() {
        anyhow::bail!(
            "Could not find a desktop entry for '{}' and no --exec was provided.",
            query
        );
    }

    wm::run_or_raise(&app.class, &ShellCommand::from(app.exec.to_string()))?;
    Ok(())
}

fn send_command(cmd: &str) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
        anyhow::anyhow!(
            "Failed to connect to halo daemon at {}: {}. Is halo running?",
            SOCKET_PATH,
            e
        )
    })?;

    writeln!(stream, "{}", cmd)?;
    Ok(())
}
