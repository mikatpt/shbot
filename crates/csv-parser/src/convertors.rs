use crate::{FilmInput, FilmOutput, StudentInput, StudentOutput};
use models::{Film, Student};

impl From<FilmInput> for Film {
    fn from(f: FilmInput) -> Film {
        Self {
            name: f.code,
            priority: f.priority,
            group_number: f.group,
            ..Default::default()
        }
    }
}

impl From<StudentInput> for Student {
    fn from(s: StudentInput) -> Self {
        Self {
            name: format!("{} {}", s.first, s.last),
            group_number: s.group,
            class: s.class,
            ..Default::default()
        }
    }
}

impl From<Student> for StudentOutput {
    fn from(s: Student) -> Self {
        let (f, l) = s.name.split_once(' ').unwrap_or_default();
        Self {
            first: f.to_string(),
            last: l.to_string(),
            class: s.class,
            group: s.group_number,
            ae: s.roles.ae.unwrap_or_default(),
            sound: s.roles.sound.unwrap_or_default(),
            editor: s.roles.editor.unwrap_or_default(),
            finish: s.roles.finish.unwrap_or_default(),
        }
    }
}

impl From<Film> for FilmOutput {
    fn from(f: Film) -> Self {
        Self {
            code: f.name,
            group: f.group_number,
            priority: f.priority,
            ae: f.roles.ae.unwrap_or_default(),
            sound: f.roles.sound.unwrap_or_default(),
            editor: f.roles.editor.unwrap_or_default(),
            finish: f.roles.finish.unwrap_or_default(),
        }
    }
}
