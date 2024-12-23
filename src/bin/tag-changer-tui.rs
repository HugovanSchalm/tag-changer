use std::{fs::File, path::PathBuf};

use clap::{builder::StringValueParser, Arg, Command};
use ratatui::{crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame};
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

#[derive(Debug, Default)]
struct App<T: Tag> {
    file: PathBuf,
    tag: T,
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
            .map(|field| Line::from(format!("{}", field)))
            .collect();
        let text = Text::from(field_lines);
        Paragraph::new(text)
            .block(block)
            .render(area, buf);
    }
}

impl <T: Tag> App<T> {
    fn new(file: PathBuf, tag: T) -> Self {
        App {
            file,
            tag,
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
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}