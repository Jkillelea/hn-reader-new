#![allow(dead_code)]
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::error;

// HTTPS network stuff
extern crate reqwest;
extern crate url;
use url::Url;

// JSON parsing
extern crate serde;
extern crate serde_json;

// Asynchronously filled buffer of the requested data from HN
pub mod article;
use article::*;

fn main() -> Result<(), Box<error::Error>> {
    let     base     = Url::parse(BASE_URL)?;
    let     top      = base.join("/v0/topstories.json")?;
    let     client   = reqwest::Client::new();
    let mut response = client.get(top).send()?;
    let mut body     = String::new();
    response.read_to_string(&mut body)?;
    let vals = char_delimited_string_to_array(&body).unwrap();

    let mut article_buffer = ArticleBuffer::new(vals);

    while let Some(art) = article_buffer.next() {
        match art {
            ArticleResult::Text(text) => {
                println!("{}", text);
            },
            ArticleResult::Waiting => {
                println!("Waiting");
                thread::sleep(Duration::from_millis(100));
            },
        }
    }

    Ok(())
}

fn char_delimited_string_to_array(string: &String) -> Result<Vec<i32>, Box<error::Error>> {
    let vals = string.split(",").map(|x| {
        x.parse().unwrap_or_else(|_| {
            if x.clone().chars().next().unwrap().is_numeric() { // last char is non-numeric
                let n = &x[0 .. x.len()-1]; // everything except last char
                n.parse().unwrap()
            } else if x.clone().chars().last().unwrap().is_numeric() { // first char is non-numeric
                let n = &x[1 .. x.len()]; // everything except first char
                n.parse().unwrap()
            } else {
                9999999
            }
        })
    }).collect();
    Ok(vals)
}
