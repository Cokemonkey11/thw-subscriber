use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{Block, Borders, Gauge, List, Paragraph, Row, Table, Tabs, Text},
    Frame,
};

use std::time::Instant;

use crate::demo::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let tab_chunks = Layout::default()
        .constraints(vec![Constraint::Length(30), Constraint::Min(10)])
        .direction(Direction::Horizontal)
        .split(chunks[0]);
    let tabs = Tabs::default()
        .block(Block::default().borders(Borders::ALL).title(app.title))
        .titles(&app.tabs.titles)
        .style(Style::default().fg(Color::Green))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, tab_chunks[0]);

    let label = format!(
        "Reload in {:.0}s",
        (app.next_update - Instant::now()).as_secs() + 1
    );
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Gray)
                .bg(Color::Black)
                .modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .label(&label)
        .ratio(app.progress);
    f.render_widget(gauge, tab_chunks[1]);

    match app.tabs.index {
        0 => draw_first_tab(f, app, chunks[1]),
        1 => draw_second_tab(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Min(11), Constraint::Length(7)].as_ref())
        .split(area);
    draw_charts(f, app, chunks[0]);
    draw_text(f, chunks[1], app);
}

fn draw_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        // Draw tasks
        let tasks = app
            .tasks
            .items
            .iter()
            .rev()
            .map(|dat| format!("{} :: {}", dat.forum, dat.title))
            .map(|i| Text::raw(i));
        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("Topics"))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);
    }
}

// Footer
fn draw_text<B>(f: &mut Frame<B>, area: Rect, app: &mut App)
where
    B: Backend,
{
    let text: Vec<Text> = match app.errors.len() {
        0 => match app.tasks.state.selected() {
            Some(idx) => vec![app
                .tasks
                .items
                .iter()
                .nth(idx)
                .map(|thw| thw.href.clone())
                .expect("task not present")]
            .into_iter()
            .map(|href| format!("https://www.hiveworkshop.com/{}", href))
            .collect(),
            _ => vec!["Select a thread with the arrow keys".into()],
        }
        .into_iter()
        .map(|str| Text::raw(str))
        .collect(),
        _ => app
            .errors
            .iter()
            .map(|str| Text::styled(str, Style::new().bg(Color::Magenta)))
            .collect(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().modifier(Modifier::HIDDEN));
    let paragraph = Paragraph::new(text.iter()).block(block).wrap(true);
    f.render_widget(paragraph, area);
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    let up_style = Style::default().fg(Color::Green);
    let failure_style = Style::default()
        .fg(Color::Red)
        .modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);
    let header = ["Server", "Location", "Status"];
    let rows = app.servers.iter().map(|s| {
        let style = if s.status == "Up" {
            up_style
        } else {
            failure_style
        };
        Row::StyledData(vec![s.name, s.location, s.status].into_iter(), style)
    });
    let table = Table::new(header.iter(), rows)
        .block(Block::default().title("Servers").borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ]);
    f.render_widget(table, chunks[0]);

    let map = Canvas::default()
        .block(Block::default().title("World").borders(Borders::ALL))
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
                color: Color::Yellow,
            });
            for (i, s1) in app.servers.iter().enumerate() {
                for s2 in &app.servers[i + 1..] {
                    ctx.draw(&Line {
                        x1: s1.coords.1,
                        y1: s1.coords.0,
                        y2: s2.coords.0,
                        x2: s2.coords.1,
                        color: Color::Yellow,
                    });
                }
            }
            for server in &app.servers {
                let color = if server.status == "Up" {
                    Color::Green
                } else {
                    Color::Red
                };
                ctx.print(server.coords.1, server.coords.0, "X", color);
            }
        })
        .marker(if app.enhanced_graphics {
            symbols::Marker::Braille
        } else {
            symbols::Marker::Dot
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    f.render_widget(map, chunks[1]);
}
