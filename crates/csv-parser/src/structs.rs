use models::Priority;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "UPPERCASE")]
/// Expected CSV input, to insert into the database.
pub struct FilmInput {
    pub code: String,
    pub group: i32,
    pub priority: Priority,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
/// Expected CSV input, to insert into the database.
pub struct StudentInput {
    pub class: String,
    pub group: i32,
    pub first: String,
    pub last: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
/// Output list of these into a csv to post.
pub struct StudentOutput {
    pub class: String,
    pub group: i32,
    pub first: String,
    pub last: String,
    // These fields are film names.
    pub ae: String,
    pub sound: String,
    pub editor: String,
    pub finish: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
/// Output list of these into a csv to post.
pub struct FilmOutput {
    pub code: String,
    pub group: i32,
    pub priority: Priority,
    // These fields are student names.
    pub ae: String,
    pub sound: String,
    pub editor: String,
    pub finish: String,
}
