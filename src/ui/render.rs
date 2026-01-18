use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[0]);

    render_request_list(frame, app, main_chunks[0]);
    render_request_details(frame, app, main_chunks[1]);
    render_status_bar(frame, app, chunks[1]);
}

fn render_request_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .requests
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let method_color = match req.method.as_str() {
                "GET" => Color::Green,
                "POST" => Color::Blue,
                "PUT" => Color::Yellow,
                "DELETE" => Color::Red,
                "PATCH" => Color::Magenta,
                _ => Color::White,
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{} ", req.timestamp_display()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:6} ", req.method),
                    Style::default().fg(method_color),
                ),
                Span::styled(req.path.clone(), style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Requests ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);

    // Render selection indicator
    if !app.requests.is_empty() && app.selected_index < app.requests.len() {
        let y = area.y + 1 + app.selected_index as u16;
        if y < area.y + area.height - 1 {
            frame.render_widget(
                Paragraph::new(">").style(Style::default().fg(Color::Yellow)),
                Rect::new(area.x + 1, y, 1, 1),
            );
        }
    }
}

fn render_request_details(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Request Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(request) = app.selected_request() {
        let mut lines: Vec<Line> = Vec::new();

        // Timestamp
        lines.push(Line::from(vec![
            Span::styled("Timestamp: ", Style::default().fg(Color::Gray)),
            Span::styled(
                request.timestamp.to_rfc3339(),
                Style::default().fg(Color::White),
            ),
        ]));

        // Method and Path
        let method_color = match request.method.as_str() {
            "GET" => Color::Green,
            "POST" => Color::Blue,
            "PUT" => Color::Yellow,
            "DELETE" => Color::Red,
            "PATCH" => Color::Magenta,
            _ => Color::White,
        };

        lines.push(Line::from(vec![
            Span::styled("Method: ", Style::default().fg(Color::Gray)),
            Span::styled(&request.method, Style::default().fg(method_color)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(request.full_path(), Style::default().fg(Color::White)),
        ]));

        lines.push(Line::from(""));

        // Headers
        lines.push(Line::from(Span::styled(
            "Headers:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        let mut headers: Vec<_> = request.headers.iter().collect();
        headers.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in headers {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", key), Style::default().fg(Color::Yellow)),
                Span::styled(value, Style::default().fg(Color::White)),
            ]));
        }

        lines.push(Line::from(""));

        // Body
        lines.push(Line::from(Span::styled(
            "Body:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(body) = request.formatted_body() {
            let body_lines: Vec<&str> = body.lines().collect();
            let max_lines = if app.body_expanded {
                body_lines.len()
            } else {
                20
            };

            for (i, line) in body_lines.iter().take(max_lines).enumerate() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", line),
                    Style::default().fg(Color::Green),
                )));

                if !app.body_expanded && i == 19 && body_lines.len() > 20 {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  ... ({} more lines, press Enter to expand)",
                            body_lines.len() - 20
                        ),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  (empty)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Apply scroll offset
        let visible_lines: Vec<Line> = lines.into_iter().skip(app.detail_scroll).collect();

        let paragraph = Paragraph::new(Text::from(visible_lines)).wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner_area);
    } else {
        let empty_text =
            Paragraph::new("No request selected").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_text, inner_area);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = format!(
        " Listening on {} | {} request{} | q: quit | c: clear | j/k: navigate | Enter: expand",
        app.listening_address,
        app.requests.len(),
        if app.requests.len() == 1 { "" } else { "s" }
    );

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}
