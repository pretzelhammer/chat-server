use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use futures::{SinkExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Terminal;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::Rotation;
use std::borrow::Cow;
use std::io;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui_textarea::{Input, Key, TextArea};
use chat_server::{connection_refused, parse_socket_addr, file_logging};

// i quickly threw this code together
// it's not particularly clean

fn textarea_new() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Start typing...");
    textarea.set_block(
        Block::default().borders(Borders::ALL).title("Send message"),
    );
    textarea
}

fn messages_to_list(
    msgs: &[String],
    min_lines: usize,
    max_length: usize,
) -> List<'_> {
    let mut list_items = Vec::new();
    // only interested in most recent msgs
    'outer: for msg in msgs.iter().rev() {
        let user_msg = msg.contains(':');
        let lines = textwrap::wrap(
            msg,
            textwrap::Options::new(max_length)
                .wrap_algorithm(textwrap::WrapAlgorithm::new_optimal_fit()),
        );
        let mut styled_lines = Vec::new();
        if user_msg {
            let mut lines = lines.into_iter();
            let first_line = lines.next().unwrap();
            let mut parts = first_line.split(':');
            let mut first_styled_line = Vec::new();
            first_styled_line.push(parts.next().unwrap().to_owned().bold());
            for part in parts {
                first_styled_line.push(Span::raw(":"));
                first_styled_line.push(part.to_owned().into());
            }
            styled_lines.push(Line::from(first_styled_line));
            for line in lines {
                styled_lines.push(Line::from(line.into_owned()));
            }
        } else {
            styled_lines.extend(
                lines
                    .into_iter()
                    .map(|line| line.into_owned().dim().italic().into()),
            );
        }
        for line in styled_lines.into_iter().rev() {
            list_items.push(ListItem::new(line));
            if list_items.len() >= min_lines {
                break 'outer;
            }
        }
    }
    // pad with empty lines
    while list_items.len() < min_lines {
        list_items.push(ListItem::new(Cow::from("")));
    }
    list_items.reverse();
    List::new(list_items)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = parse_socket_addr();
    let mut conn = match TcpStream::connect(addr).await {
        Ok(conn) => conn,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::ConnectionRefused => {
                    println!("{}", connection_refused(addr));
                    std::process::exit(1)
                }
                // got unexpected err, re-throw
                _ => Err(err)?,
            }
        }
    };

    let (reader, writer) = conn.split();
    let mut tcp_sink = FramedWrite::new(writer, LinesCodec::new());
    let mut tcp_stream = FramedRead::new(reader, LinesCodec::new());

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut _guard: Option<WorkerGuard> = None;

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = textarea_new();
    let layout = Layout::default()
        .constraints([Constraint::Percentage(100), Constraint::Min(3)]);

    let mut messages: Vec<String> = Vec::new();
    let mut current_room = "main".to_owned();

    let mut term_stream = crossterm::event::EventStream::new();

    loop {
        let draw_res = term.draw(|f| {
            let chunks = layout.split(f.size());

            let msgs_height = chunks[0].height - 2; // -2 for borders
            let msgs_width = chunks[0].width - 2; // -2 for borders
            let msgs_title = format!("Room - {current_room}");
            let msgs = messages_to_list(
                &messages,
                msgs_height.into(),
                msgs_width.into(),
            )
            .block(Block::default().borders(Borders::ALL).title(msgs_title));
            f.render_widget(msgs, chunks[0]);

            // render input box
            let widget = textarea.widget();
            f.render_widget(widget, chunks[1]);
        });

        match draw_res {
            Ok(_) => (),
            Err(_) => break,
        };

        tokio::select! {
            term_event = term_stream.next() => {
                if let Some(event) = term_event {
                    let event = match event {
                        Ok(event) => event,
                        Err(_) => break,
                    };
                    match event.into() {
                        // escape
                        Input { key: Key::Esc, .. } |
                        // ctrl+c
                        Input { key: Key::Char('c'), ctrl: true, .. } |
                        // ctrl+d
                        Input { key: Key::Char('d'), ctrl: true, .. }  => break,
                        // enter
                        Input { key: Key::Enter, .. } => {
                            if textarea.is_empty() {
                                continue;
                            }
                            //messages.extend(textarea.into_lines());
                            for line in textarea.into_lines() {
                                tracing::info!("SENT {line}");
                                match tcp_sink.send(line).await {
                                    Ok(_) => (),
                                    Err(_) => break,
                                };
                            }
                            textarea = textarea_new();
                        }
                        // forward input to textarea
                        input => {
                            // messages.push(format!("{:?}", input));
                            // TextArea::input returns if the input modified its text
                            textarea.input_without_shortcuts(input);
                        }
                    }
                } else {
                    break;
                }
            },
            tcp_event = tcp_stream.next() => match tcp_event {
                Some(event) => {
                    let server_msg = match event {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };
                    if server_msg.starts_with("You joined ") {
                        let room_name = server_msg
                            .split_ascii_whitespace()
                            .nth(2)
                            .unwrap();
                        current_room = room_name.to_owned();
                    } else if server_msg.starts_with("You are ") {
                        let name = server_msg
                            .split_ascii_whitespace()
                            .nth(2)
                            .unwrap();
                        _guard = Some(file_logging(Rotation::NEVER, &format!("chat-tui.{name}.log")));
                    }
                    tracing::info!(" GOT {server_msg}");
                    messages.push(server_msg);
                },
                None => break,
            },
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(term.backend_mut(), LeaveAlternateScreen,)?;
    term.show_cursor()?;
    Ok(())
}
