#[allow(dead_code)]
mod demo;
#[allow(dead_code)]
mod util;

use crate::demo::{ui, App, ThwDatum};
use argh::FromArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use select::document::Document;
use select::predicate::{Class, Predicate};

use std::{
    error::Error,
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

enum Event<I> {
    Input(I),
    Tick,
}

/// Crossterm demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(cli.tick_rate);
    let handle_in = thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            if event::poll(tick_rate - last_tick.elapsed()).expect("failed to poll tickrate") {
                if let CEvent::Key(key) = event::read().expect("failed to read event") {
                    tx.send(Event::Input(key))
                        .expect("failed to send on channel");
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick)
                    .expect("failed to send tick on channel");
                last_tick = Instant::now();
            }
        }
    });

    let (refresh_tx, refresh_rx) = mpsc::channel();
    let (results_tx, results_rx) = mpsc::channel();

    let tx_clone = refresh_tx.clone();
    tx_clone.send(()).expect("Failed to send initial unit");

    let mut app = App::new(
        "THW Subscriber",
        cli.enhanced_graphics,
        refresh_tx,
        results_rx,
    );

    let handle = thread::spawn(move || loop {
        match refresh_rx.recv() {
            Ok(()) => {
                let body = ureq::get("https://www.hiveworkshop.com/find-new/posts")
                    .call()
                    .into_string()
                    .expect("Failed to fetch");
                Document::from(&body[..])
                    .find(Class("titleText"))
                    .map(|node| {
                        let title = node
                            .find(Class("title").descendant(Class("PreviewTooltip")))
                            .next()
                            .expect("missing title");
                        let second_row = node
                            .find(Class("secondRow").descendant(Class("forumLink")))
                            .next()
                            .expect("missing forum");

                        (
                            title.attr("href").expect("node didn't have href").into(),
                            title.text(),
                            second_row.text(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .for_each(|(href, title, forum)| {
                        results_tx
                            .send(ThwDatum { title, forum, href })
                            .expect("failed to send test datum");
                    });
            }
            Err(e) => panic!("{:?} {:?}", e, e.to_string()),
        }
    });

    terminal.clear()?;

    loop {
        terminal.draw(|mut f| ui::draw(&mut f, &mut app))?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char(c) => app.on_key(c),
                KeyCode::Left => app.on_left(),
                KeyCode::Up => app.on_up(),
                KeyCode::Right => app.on_right(),
                KeyCode::Down => app.on_down(),
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    handle.join().expect("failed to join thread");
    handle_in.join().expect("failed to join handle-in");

    Ok(())
}
