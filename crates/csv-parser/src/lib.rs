use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod convertors;
mod structs;

use reqwest::tls::Version;
pub use structs::{FilmInput, FilmOutput, StudentInput, StudentOutput};

/// Read from a url into csv. This will error out if deserialization fails!
pub async fn from_url<'a, T: for<'de> Deserialize<'de>>(url: &'a str) -> Result<Vec<T>> {
    let token = std::env::var("OAUTH_TOKEN")?;
    let client = reqwest::Client::builder().build()?;
    let text = client
        .get(url)
        .bearer_auth(&token)
        .send()
        .await?
        .text()
        .await?;
    let bytes = text.as_str();
    dbg!(bytes);

    let mut films = vec![];
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    for result in rdr.deserialize() {
        let record: T = result?;
        films.push(record);
    }

    Ok(films)
}

/// Writes from struct into csv format.
pub fn to_csv_string<T: Serialize>(items: Vec<T>) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    for item in items {
        wtr.serialize(item)?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(data)
}

/// Uploads a CSV string to slack.
pub async fn to_slack(csv: String, title: &str, msg: &str, channel: &str) -> Result<()> {
    let token = std::env::var("OAUTH_TOKEN")?;
    let url = "https://slack.com/api/files.upload";

    let mut params = HashMap::new();
    params.insert("filetype", "csv");
    params.insert("title", title);
    params.insert("content", &csv);
    params.insert("channels", channel);
    params.insert("initial_comment", msg);
    let version = Version::TLS_1_2;

    let client = reqwest::Client::builder()
        .min_tls_version(version)
        .build()?;
    client
        .post(url)
        .bearer_auth(token)
        .form(&params)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "hits dropbox"]
    async fn test_read_film() -> Result<()> {
        let url = "https://www.dropbox.com/s/n41glq2a79v1xtj/sample_film_input.csv?dl=1";

        let text: Vec<FilmInput> = from_url(url).await?;

        dbg!(text);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "test slack"]
    async fn test_read() -> Result<()> {
        dotenv::dotenv().ok();
        let url = "https://files.slack.com/files-pri/T038MGR9PDH-F039QA7KDLN/sample_film_input.csv";
        let text: Vec<FilmInput> = from_url(url).await?;
        dbg!(text);
        Ok(())
    }

    #[test]
    fn test_write() -> Result<()> {
        let s = StudentOutput {
            class: "a".to_string(),
            group: 1,
            first: "a".to_string(),
            last: "z".to_string(),
            ae: "a".to_string(),
            sound: "a".to_string(),
            editor: "a".to_string(),
            finish: "a".to_string(),
        };

        let contents = to_csv_string(vec![s])?;

        let has_header = contents.contains("CLASS,GROUP,FIRST,LAST,AE,SOUND,EDITOR,FINISH");
        assert!(has_header);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Hits slack"]
    async fn test_slack() -> Result<()> {
        dotenv::dotenv().ok();
        let s = StudentOutput {
            class: "a".to_string(),
            group: 1,
            first: "a".to_string(),
            last: "z".to_string(),
            ae: "a".to_string(),
            sound: "a".to_string(),
            editor: "a".to_string(),
            finish: "a".to_string(),
        };

        let contents = to_csv_string(vec![s])?;
        to_slack(contents, "test", "Hey", "U038MGZT5T4").await?;

        Ok(())
    }
}
