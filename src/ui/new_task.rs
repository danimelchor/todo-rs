use crate::app::App;
use crate::{repeat::Repeat, task::Task};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use crossterm::event::{self, Event, KeyCode};
use std::{cell::RefCell, rc::Rc};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use super::{Page, UIPage};

pub struct TaskForm {
    pub name: String,
    pub date: String,
    pub repeats: String,
    pub description: String,
}

impl TaskForm {
    pub fn new() -> TaskForm {
        TaskForm {
            name: "".to_string(),
            date: "".to_string(),
            repeats: "".to_string(),
            description: "".to_string(),
        }
    }

    pub fn submit(&mut self) -> Result<Task> {
        let mut task = Task::new();

        let repeat = Repeat::parse_from_str(&self.repeats).context("Invalid repeat format")?;
        let date =
            NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").context("Invalid date format")?;

        task.set_name(self.name.clone());
        task.set_date(date);
        task.set_repeats(repeat);
        task.set_description(self.description.clone());

        Ok(task)
    }
}

#[derive(PartialEq)]
pub enum NewTaskInputMode {
    Normal,
    Editing,
}

pub struct NewTaskPage {
    pub task_form: TaskForm,
    pub input_mode: NewTaskInputMode,
    pub current_idx: usize,
    pub num_fields: usize,
    pub error: Option<String>,
    pub app: Rc<RefCell<App>>,
}

impl NewTaskPage {
    pub fn new(app: Rc<RefCell<App>>) -> NewTaskPage {
        NewTaskPage {
            task_form: TaskForm::new(),
            input_mode: NewTaskInputMode::Normal,
            current_idx: 0,
            error: None,
            num_fields: 4,
            app,
        }
    }

    pub fn next_field(&mut self) {
        if self.current_idx < self.num_fields - 1 {
            self.current_idx += 1;
        }
    }

    pub fn prev_field(&mut self) {
        if self.current_idx > 0 {
            self.current_idx -= 1;
        }
    }

    pub fn add_char(&mut self, c: char) {
        match self.current_idx {
            0 => {
                self.task_form.name.push(c);
            }
            1 => {
                self.task_form.date.push(c);
            }
            2 => {
                self.task_form.repeats.push(c);
            }
            3 => {
                self.task_form.description.push(c);
            }
            _ => {}
        };
    }

    pub fn remove_char(&mut self) {
        match self.current_idx {
            0 => {
                self.task_form.name.pop();
            }
            1 => {
                self.task_form.date.pop();
            }
            2 => {
                self.task_form.repeats.pop();
            }
            3 => {
                self.task_form.description.pop();
            }
            _ => {}
        };
    }

    fn border_style(&self, idx: usize) -> Style {
        if self.current_idx == idx && self.input_mode == NewTaskInputMode::Editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        }
    }
}

impl<B> Page<B> for NewTaskPage
where
    B: Backend,
{
    fn render(&mut self, terminal: &mut Terminal<B>) -> Result<UIPage> {
        terminal.draw(|f| self.ui(f))?;

        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                NewTaskInputMode::Normal => match key.code {
                    KeyCode::Char('j') => self.next_field(),
                    KeyCode::Char('k') => self.prev_field(),
                    KeyCode::Char('q') => {
                        return Ok(UIPage::Quit);
                    }
                    KeyCode::Char('i') => {
                        self.input_mode = NewTaskInputMode::Editing;
                    }
                    KeyCode::Char('b') => {
                        return Ok(UIPage::AllTasks);
                    }
                    KeyCode::Enter => {
                        match self.task_form.submit() {
                            Ok(new_taks) => {
                                self.app.borrow_mut().add_task(new_taks);
                                return Ok(UIPage::AllTasks);
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                            }
                        }
                    },
                    _ => {}
                },
                _ => match key.code {
                    KeyCode::Esc => {
                        self.input_mode = NewTaskInputMode::Normal;
                    }
                    KeyCode::Char(c) => self.add_char(c),
                    KeyCode::Backspace => self.remove_char(),
                    _ => {}
                },
            }
        }
        Ok(UIPage::SamePage)
    }

    fn ui(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        // Keybinds description paragraph
        let keybinds = Paragraph::new(
        "Press 'i' to enter input mode, 'q' to quit, 'j' and 'k' to move up and down, 'Enter' to submit, 'Esc' to exit input mode, and 'b' to go back to the main screen"
    ).alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        f.render_widget(keybinds, chunks[0]);

        // Name
        let curr_text = self.task_form.name.clone();
        let input = Paragraph::new(curr_text.as_ref())
            .style(self.border_style(0))
            .block(Block::default().borders(Borders::ALL).title("Name"));
        f.render_widget(input, chunks[1]);

        // Date
        let curr_text = self.task_form.date.clone();
        let input = Paragraph::new(curr_text.as_ref())
            .style(self.border_style(1))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Date (YYYY-MM-DD)"),
            );
        f.render_widget(input, chunks[2]);

        // Repeats
        let curr_text = self.task_form.repeats.to_string();
        let input = Paragraph::new(curr_text.as_ref())
            .style(self.border_style(2))
            .block(Block::default().borders(Borders::ALL).title(
                "Repeats (Never | Daily | Weekly | Monthly | Yearly | Mon,Tue,Wed,Thu,Fri,Sat,Sun)",
            ));
        f.render_widget(input, chunks[3]);

        // Description
        let curr_text = self.task_form.description.clone();
        let input = Paragraph::new(curr_text.as_ref())
            .style(self.border_style(3))
            .block(Block::default().borders(Borders::ALL).title("Description"));
        f.render_widget(input, chunks[4]);

        // Place cursor
        match self.current_idx {
            0 => f.set_cursor(
                chunks[1].x + self.task_form.name.width() as u16 + 1,
                chunks[1].y + 1,
            ),
            1 => f.set_cursor(
                chunks[2].x + self.task_form.date.width() as u16 + 1,
                chunks[2].y + 1,
            ),
            2 => f.set_cursor(
                chunks[3].x + self.task_form.repeats.width() as u16 + 1,
                chunks[3].y + 1,
            ),
            3 => f.set_cursor(
                chunks[4].x + self.task_form.description.width() as u16 + 1,
                chunks[4].y + 1,
            ),
            _ => {}
        }

        // Error message
        if let Some(error) = &self.error {
            let error = Paragraph::new(error.as_ref())
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Error"));
            f.render_widget(error, chunks[4]);
        }
    }
}
