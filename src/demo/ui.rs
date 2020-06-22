use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
            .map(|dat| format!("{} :: {}", dat.forum, dat.title))
            .map(Text::raw);
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
        0 => {
            vec![Text::raw(app.get_uri().unwrap_or_else(|| {
                "Select a thread with the arrow keys".into()
            }))]
        }
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

    let chunks = Layout::default()
        .constraints([Constraint::Min(40), Constraint::Length(10)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    f.render_widget(paragraph, chunks[0]);

    let legend_text = vec![
        Text::styled("Q", Style::new().modifier(Modifier::UNDERLINED)),
        Text::raw("uit\n"),
        Text::styled("C", Style::new().modifier(Modifier::UNDERLINED)),
        Text::raw("opy url\n"),
    ];
    let legend = Paragraph::new(legend_text.iter()).block(Block::default().borders(Borders::ALL));
    f.render_widget(legend, chunks[1]);
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let blocked_style = Style::default().fg(Color::Red);
    let header = ["Forum", "Status"];
    let rows = app.filters.iter().map(|s| {
        Row::StyledData(
            vec![s.to_string(), "Blocked".to_string()].into_iter(),
            blocked_style,
        )
    });
    let table = Table::new(header.iter(), rows)
        .block(Block::default().borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[Constraint::Length(25), Constraint::Length(20)]);
    f.render_widget(table, area);
}
