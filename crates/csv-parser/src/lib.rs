use color_eyre::Result;
use serde::{Deserialize, Serialize};

/*
Main goals: all we do is handle in-out.


Sheree can upload a csv doc to upload all films:

# films.csv
group,film_code,priority

# students.csv
class,group,first,last

There are 9 groups, each with 6 films.
*/

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub struct FilmInput {
    pub code: String,
    pub group: i32,
    pub priority: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct StudentsInput {
    pub class: String,
    pub group: i32,
    pub first: String,
    pub last: String,
}

pub async fn read_film_csv(url: &str) -> Result<Vec<FilmInput>> {
    let text = reqwest::get(url).await?.text().await?;
    dbg!(&text);

    let mut films = vec![];
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    for result in rdr.deserialize() {
        let record: FilmInput = result?;
        films.push(record);
    }

    Ok(films)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_read_film() -> Result<()> {
        let url = "https://www.dropbox.com/s/n41glq2a79v1xtj/sample_film_input.csv?dl=1";

        let text = read_film_csv(url).await?;

        dbg!(text);

        Ok(())
    }
}
