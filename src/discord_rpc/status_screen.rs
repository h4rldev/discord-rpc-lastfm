use super::Presence;
use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveTo},
    style::Print,
    terminal::{size, Clear, ClearType, SetTitle},
    ExecutableCommand,
};
use std::io::{stdout, Stdout};
use tokio::time::{sleep, Duration};

fn print_newline(stdout: &mut Stdout, count: u16) -> Result<(), std::io::Error> {
    for _ in 0..count {
        stdout.execute(Print("\n"))?;
    }
    Ok(())
}

fn clear_screen(stdout: &mut Stdout) -> Result<(), std::io::Error> {
    stdout
        .execute(MoveTo(0, 0))?
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(0, 0))?;
    Ok(())
}

pub async fn status_screen(presence: Presence) -> tokio::io::Result<()> {
    let mut stdout = stdout();
    stdout
        .execute(Hide)?
        .execute(SetTitle("discord-rpc-lastfm"))?;
    clear_screen(&mut stdout)?;

    let output = format!(
        "{}\n{}\n{}",
        presence.details.green(),
        presence.state.red(),
        presence.large_text.yellow(),
    );
    let (width, height) = size()?;
    let lines = output.matches('\n').count() + 1;
    let lines = lines as u16;
    let vertical_padding = if height > lines {
        (height - lines) / 2
    } else {
        0
    };
    print_newline(&mut stdout, vertical_padding)?;
    let output_lines: Vec<&str> = output.lines().collect();
    for line in output_lines {
        stdout.execute(Print(format!("{:^width$}\n", line, width = width as usize)))?;
    }
    print_newline(&mut stdout, vertical_padding)?;
    stdout.execute(MoveTo(0, 0))?;
    sleep(Duration::from_secs_f32(0.5)).await;
    Ok(())
}
