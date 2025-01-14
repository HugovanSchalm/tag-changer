use std::{fs::File, path::PathBuf};

use clap::{builder::StringValueParser, Arg, Command};
use ratatui::prelude::*;
use ratatui::{crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, style::{Color, Style}, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame};
use tag_changer::{ID3v1, Tag};

fn main() -> std::io::Result<()> {
    let matches = Command::new("tag-changer-tui")
        .arg(Arg::new("file")
            .required(true)
            .value_parser(StringValueParser::new())
        ).get_matches();
    let file: &String = matches.get_one("file").unwrap();
    let filepath = PathBuf::from(file);
    let mut file = File::open(filepath.clone()).unwrap();
    let tag = ID3v1::read(&mut file);
    let tag = match tag {
        Ok(valid_tag) => valid_tag,
        Err(_) => ID3v1::default(),
    };
    let mut terminal = ratatui::init();
    App::new(filepath, tag).run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

#[derive(Debug)]
enum AppState {
    Viewing,
    Editing(usize),
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Viewing
    }
}

#[derive(Debug, Default)]
struct App<T: Tag> {
    file: PathBuf,
    state: AppState,
    tag: T,
    selected_field_index: usize,
    should_exit: bool,
}

impl <T: Tag> Widget for &App<T> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        let title  = Line::from(self.file.file_name().unwrap().to_str().unwrap())
                .centered();
        let instructions = Line::from("<UP><DOWN> to select | <ENTER> to edit | <q> to exit")
                .centered();
        let block = Block::bordered()
                .title_top(title)
                .title_bottom(instructions);
        let field_lines: Vec<Line> = self
            .tag
            .get_fields()
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let mut line = Line::from(format!("{}", field));
                if i == self.selected_field_index {
                    let selected_style = Style::new().bg(Color::Yellow);
                    line = line.style(selected_style);
                }
                line
            })
            .collect();

        let text = Text::from(field_lines);
        Paragraph::new(text)
            .block(block)
            .render(area, buf);
        
    }
}

/// Source: https://ratatui.rs/tutorials/json-editor/ui/
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

impl <T: Tag> App<T> {
    fn new(file: PathBuf, tag: T) -> Self {
        App {
            file,
            state: AppState::Viewing,
            tag,
            selected_field_index: 0,
            should_exit: false,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        if let AppState::Editing(i) = self.state {
            let field = &self.tag.get_fields()[i];
            let popup = Block::bordered()
                .title(field.get_name());
            let text = Paragraph::new(Text::from("Hoi"))
                .block(popup);

            let area = centered_rect(60, 40, frame.area());
            frame.render_widget(text, area);
        } 
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            AppState::Viewing => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Up => self.select_previous_field(),
                    KeyCode::Down => self.select_next_field(),
                    KeyCode::Enter => self.state = AppState::Editing(self.selected_field_index),
                    _ => {}
                }
            }
            AppState::Editing(_) => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Esc => self.state = AppState::Viewing,
                    _ => {}
                }
            }
        }
    }
    
    fn select_next_field(&mut self) {
        if self.selected_field_index >= self.tag.get_field_count() - 1 {
            self.selected_field_index = 0
        } else {
            self.selected_field_index += 1;
        }
    }
    
    fn select_previous_field(&mut self) {
        if self.selected_field_index == 0 {
            self.selected_field_index = self.tag.get_field_count() - 1
        } else {
            self.selected_field_index -= 1;
        }
    }


    fn exit(&mut self) {
        self.should_exit = true;
    }
}
