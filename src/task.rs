use crate::{day_of_week::DayOfWeek, repeat::Repeat};
use anyhow::Result;
use chrono::{Datelike, Days, Local, Months, DateTime, TimeZone};
use serde::{Deserialize, Serialize};

pub fn serialize_dt<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = date.format("%+").to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize_dt<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = Local.datetime_from_str(&s, "%+").unwrap();
    Ok(dt)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: Option<usize>,
    pub name: String,
    #[serde(serialize_with = "serialize_dt", deserialize_with = "deserialize_dt")]
    pub date: DateTime<Local>,
    pub repeats: Repeat,
    pub description: Option<String>,
    pub complete: bool,
}

impl Task {
    pub fn new() -> Task {
        Task {
            id: None,
            name: "".to_string(),
            date: Local::now(),
            repeats: Repeat::Never,
            description: None,
            complete: false,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_date(&mut self, date: DateTime<Local>) {
        self.date = date;
    }

    pub fn set_repeats(&mut self, repeats: Repeat) {
        self.repeats = repeats;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_complete(&mut self) -> Option<Task> {
        self.complete = true;
        let date = match &self.repeats {
            Repeat::DaysOfWeek(days) => {
                let mut new_date = None;
                for i in 1..=7 {
                    let day = self.date.checked_add_days(Days::new(i)).unwrap();
                    let weekday = DayOfWeek::from_chrono(day.weekday());
                    if days.contains(&weekday) {
                        new_date = self.date.checked_add_days(Days::new(i));
                        break;
                    }
                }
                new_date
            }
            Repeat::Never => None,
            Repeat::Daily => self.date.checked_add_days(Days::new(1)),
            Repeat::Weekly => self.date.checked_add_days(Days::new(7)),
            Repeat::Monthly => self.date.checked_add_months(Months::new(1)),
            Repeat::Yearly => self.date.checked_add_months(Months::new(12)),
        };

        if let Some(date) = date {
            let mut new_task = self.clone();
            new_task.set_date(date);
            new_task.set_incomplete();
            Some(new_task)
        } else {
            None
        }
    }

    pub fn set_incomplete(&mut self) -> Option<Task> {
        self.complete = false;
        None
    }

    pub fn toggle_complete(&mut self) -> Option<Task> {
        if self.complete {
            self.set_incomplete()
        } else {
            self.set_complete()
        }
    }

    pub fn get_id(&self) -> usize {
        match self.id {
            Some(id) => id,
            None => panic!("Tasks should always have an ID once added"),
        }
    }
}
