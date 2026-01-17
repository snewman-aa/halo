use clap::Parser;
use serde::Deserialize;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The window class to search for (e.g., "vesktop")
    class: String,

    /// The command to execute if the window is not found (typically launching)
    #[arg(short = 'e', long)]
    exec: String,
}

#[derive(Deserialize, Debug)]
struct Client {
    class: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let output = Command::new("hyprctl").arg("clients").arg("-j").output()?;

    let clients: Vec<Client> = serde_json::from_slice(&output.stdout)?;

    let found = clients.iter().any(|client| client.class == args.class);

    if found {
        Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("class:{}", args.class))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&args.exec)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    Ok(())
}
