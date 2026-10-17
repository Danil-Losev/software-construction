use std::io::{self, IsTerminal, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Terminal,
};

/// Definition of a table column that stretches proportionally.
#[derive(Debug, Clone)]
pub struct TerminalTableColumn {
    pub header: String,
    pub width_ratio: u32,
    pub min_width: usize,
}

/// Table data that automatically expands to fill full terminal width.
#[derive(Debug, Clone)]
pub struct TerminalTable {
    pub columns: Vec<TerminalTableColumn>,
    pub rows: Vec<Vec<String>>,
}

/// A visual section within the terminal problem display.
#[derive(Debug, Clone)]
pub struct TerminalUiSection {
    pub section_title: String,
    pub lines: Vec<Line<'static>>,
    pub table: Option<TerminalTable>,
}

impl TerminalUiSection {
    pub fn new_text(title: impl Into<String>, lines: Vec<Line<'static>>) -> Self {
        Self {
            section_title: title.into(),
            lines,
            table: None,
        }
    }

    pub fn new_table(
        title: impl Into<String>,
        columns: Vec<TerminalTableColumn>,
        rows: Vec<Vec<String>>,
    ) -> Self {
        Self {
            section_title: title.into(),
            lines: Vec::new(),
            table: Some(TerminalTable { columns, rows }),
        }
    }
}

/// Complete presentation data model for the terminal UI viewer.
#[derive(Debug, Clone)]
pub struct TerminalUiProblemView {
    pub format_display_name: String,
    pub problem_title: String,
    pub summary_statistics: Vec<Line<'static>>,
    pub sections: Vec<TerminalUiSection>,
}

/// Runs an interactive terminal UI for viewing problem details.
/// Falls back to standard console output if running in a non-interactive environment (CI, redirected stdout).
pub fn render_interactive_problem_view(view: &TerminalUiProblemView) {
    if !io::stdout().is_terminal() {
        render_headless_fallback(view);
        return;
    }

    if let Err(e) = run_tui_app(view) {
        eprintln!("Error during interactive terminal UI: {}", e);
        render_headless_fallback(view);
    }
}

fn run_tui_app(view: &TerminalUiProblemView) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = event_loop(&mut terminal, view);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    view: &TerminalUiProblemView,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut scroll_offset: u16 = 0;

    loop {
        let size = terminal.size()?;
        let content_width = (size.width as usize).saturating_sub(6).max(30);

        // Generate lines dynamically to fill 100% of terminal width
        let all_lines = build_lines_for_width(view, content_width);
        let total_lines_count = all_lines.len() as u16;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header banner
                    Constraint::Min(5),    // Main scrollable content
                    Constraint::Length(3), // Footer navigation bar
                ])
                .split(f.area());

            // Header banner spanning full width
            let header_text = vec![Line::from(vec![
                Span::styled(" [SYSTEM] ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    view.format_display_name.clone(),
                    Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  │  "),
                Span::styled("TASK: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    view.problem_title.clone(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ])];

            let header_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Competitive Programming Problem Inspector ");
            let header_paragraph = Paragraph::new(header_text).block(header_block);
            f.render_widget(header_paragraph, chunks[0]);

            // Body content
            let body_height = chunks[1].height.saturating_sub(2);
            let max_scroll = total_lines_count.saturating_sub(body_height);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }

            let body_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray))
                .title(format!(
                    " Problem Specification & Tests [Line {}/{}] ",
                    scroll_offset + 1,
                    total_lines_count
                ));

            let content_paragraph = Paragraph::new(all_lines)
                .block(body_block)
                .wrap(Wrap { trim: false })
                .scroll((scroll_offset, 0));
            f.render_widget(content_paragraph, chunks[1]);

            // Scrollbar
            let mut scrollbar_state =
                ScrollbarState::new(max_scroll as usize).position(scroll_offset as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");
            f.render_stateful_widget(
                scrollbar,
                chunks[1].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );

            // Footer controls bar spanning full width
            let footer_text = vec![Line::from(vec![
                Span::styled(" [↑/k] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
                Span::raw(" Up  "),
                Span::styled(" [↓/j] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
                Span::raw(" Down  "),
                Span::styled(" [PgUp/PgDn] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
                Span::raw(" Page  "),
                Span::styled(" [Home/End] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
                Span::raw(" Extremes  "),
                Span::styled(" [q / Esc] ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(" Quit Viewer", Style::default().fg(Color::LightRed)),
            ])];
            let footer_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Controls ");
            let footer_paragraph = Paragraph::new(footer_text).block(footer_block);
            f.render_widget(footer_paragraph, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Up | KeyCode::Char('k') => {
                                scroll_offset = scroll_offset.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                scroll_offset = scroll_offset.saturating_add(1);
                            }
                            KeyCode::PageUp => {
                                scroll_offset = scroll_offset.saturating_sub(15);
                            }
                            KeyCode::PageDown => {
                                scroll_offset = scroll_offset.saturating_add(15);
                            }
                            KeyCode::Home => {
                                scroll_offset = 0;
                            }
                            KeyCode::End => {
                                scroll_offset = total_lines_count;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        scroll_offset = scroll_offset.saturating_add(3);
                    }
                    MouseEventKind::ScrollUp => {
                        scroll_offset = scroll_offset.saturating_sub(3);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(())
}

fn build_lines_for_width(view: &TerminalUiProblemView, content_width: usize) -> Vec<Line<'static>> {
    let mut all_lines: Vec<Line<'static>> = Vec::new();

    // Summary statistics block content
    if !view.summary_statistics.is_empty() {
        let title = "KEY METRICS & RESOURCE LIMITS";
        let bar_len = content_width.saturating_sub(title.len() + 6);
        let bar_str = "═".repeat(bar_len);

        all_lines.push(Line::from(vec![
            Span::styled("╔═ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}╗", bar_str), Style::default().fg(Color::Yellow)),
        ]));
        all_lines.extend(view.summary_statistics.clone());
        all_lines.push(Line::from(""));
    }

    for section in &view.sections {
        let bar_len = content_width.saturating_sub(section.section_title.len() + 6);
        let bar_str = "─".repeat(bar_len);

        all_lines.push(Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(Color::Cyan)),
            Span::styled(
                section.section_title.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}┐", bar_str), Style::default().fg(Color::Cyan)),
        ]));

        if let Some(ref table) = section.table {
            render_table_lines(table, content_width, &mut all_lines);
        } else {
            all_lines.extend(section.lines.clone());
        }

        all_lines.push(Line::from(""));
    }

    all_lines
}

fn render_table_lines(
    table: &TerminalTable,
    content_width: usize,
    output: &mut Vec<Line<'static>>,
) {
    if table.columns.is_empty() {
        return;
    }

    let col_count = table.columns.len();
    let total_ratio: u32 = table.columns.iter().map(|c| c.width_ratio).sum();
    let spacing_cost = (col_count - 1) * 2 + 2; // 2 spaces between columns
    let net_width = content_width.saturating_sub(spacing_cost);

    let mut col_widths: Vec<usize> = table
        .columns
        .iter()
        .map(|c| {
            let w = if total_ratio > 0 {
                ((net_width as u32 * c.width_ratio) / total_ratio) as usize
            } else {
                net_width / col_count
            };
            w.max(c.min_width)
        })
        .collect();

    // Adjust last column to match remaining width exactly
    let sum_first: usize = col_widths.iter().take(col_count - 1).sum();
    if let Some(last) = col_widths.last_mut() {
        let remaining = net_width.saturating_sub(sum_first);
        if remaining > *last {
            *last = remaining;
        }
    }

    // Header line
    let mut header_spans = Vec::new();
    header_spans.push(Span::raw("  "));
    for (i, col) in table.columns.iter().enumerate() {
        let width = col_widths[i];
        let text = format!("{:<width$}", col.header, width = width);
        header_spans.push(Span::styled(
            text,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
        if i + 1 < col_count {
            header_spans.push(Span::raw("  "));
        }
    }
    output.push(Line::from(header_spans));

    // Rows
    for row in &table.rows {
        let mut row_spans = Vec::new();
        row_spans.push(Span::raw("  "));
        for (i, cell) in row.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(15);
            let text = format!("{:<width$}", cell, width = width);

            let style = if i == 0 {
                Style::default().fg(Color::Yellow)
            } else if i == 1 {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::LightCyan)
            };

            row_spans.push(Span::styled(text, style));
            if i + 1 < col_count {
                row_spans.push(Span::raw("  "));
            }
        }
        output.push(Line::from(row_spans));
    }
}

fn render_headless_fallback(view: &TerminalUiProblemView) {
    let content_width = 80;
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!(
        "║ SYSTEM: {:<20} TASK: {:<38} ║",
        view.format_display_name, view.problem_title
    );
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");

    let all_lines = build_lines_for_width(view, content_width);
    for line in &all_lines {
        print_plain_line(line);
    }
}

fn print_plain_line(line: &Line<'_>) {
    let mut s = String::new();
    for span in &line.spans {
        s.push_str(&span.content);
    }
    println!("{}", s);
}
