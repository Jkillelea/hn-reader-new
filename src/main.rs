#![allow(dead_code)]
extern crate reqwest;
extern crate url;
use url::Url;

use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

const BASE_URL: &str = "https://hacker-news.firebaseio.com/";

#[derive(Debug)]
enum Article {
    Text(String), // the actual article
    Waiting,      // waiting on IO,
}

// ArticleBuffer struct holds a list of articles and allows a user to iterate over them
#[derive(Debug)]
struct ArticleBuffer {
    buffer: Arc<Mutex<Vec<String>>>,
    thread_running: Arc<Mutex<bool>>,
}

impl ArticleBuffer {
    // Create an ArticleBuffer and spawn a background thread to fill it
    fn new(ids: Vec<i32>) -> ArticleBuffer {
        let article_buffer = ArticleBuffer {
            buffer: Arc::new(Mutex::new(Vec::new())),
            thread_running: Arc::new(Mutex::new(true))
        };

        let buf = article_buffer.buffer.clone(); // cloning these only clones a *handle*
        let running = article_buffer.thread_running.clone();
        thread::spawn(move || { // Launch a new thread
            let base   = Url::parse(BASE_URL).unwrap();
            let client = reqwest::Client::new();
            for id in ids { // for each article ID
                let mut body = String::new();
                let url = base.join(&format!("/v0/item/{}.json?print=pretty", id)).unwrap(); // Generate a URL
                let mut response = client.get(url).send().unwrap(); // make the request
                response.read_to_string(&mut body).unwrap();        // read it in
                buf.lock().unwrap().push(body);                     // and add it to the list
            }
            *running.lock().unwrap() = false; // When done, let the main thread know
        });

        article_buffer
    }
}

impl Iterator for ArticleBuffer { // Loop over the content
    type Item = Article; // The type of item we'll return is a string

    // The only required method. Each time it is called, it should either return
    // the next item that we want, or None.
    fn next(&mut self) -> Option<Self::Item> {
        let article = self.buffer.lock().unwrap().pop();
        if article.is_some() {                    // See if an article is ready
            Some(Article::Text(article.unwrap())) // and if so, return it
        } else {
            if *self.thread_running.lock().unwrap() { // If we're waiting on network
                return Some(Article::Waiting);
            }
            None // Actually out of content? Return None
        }
    }
}

fn main() -> Result<(), Box<::std::error::Error>> {
    let     base     = Url::parse(BASE_URL)?;
    let     top      = base.join("/v0/topstories.json")?;
    let     client   = reqwest::Client::new();
    let mut response = client.get(top).send()?;
    let mut body     = String::new();
    response.read_to_string(&mut body)?;
    let vals = char_delimited_string_to_array(&body).unwrap();

    let mut article_buffer = ArticleBuffer::new(vals);
    thread::sleep(::std::time::Duration::from_secs(1));

    while let Some(art) = article_buffer.next() {
        match art {
            Article::Text(text) => println!("{}", text),
            Article::Waiting => {
                println!("Waiting");
                thread::sleep(::std::time::Duration::from_millis(100));
            },
        }
    }

    Ok(())
}


fn char_delimited_string_to_array(string: &String) -> Result<Vec<i32>, Box<::std::error::Error>> {
    let vals: Vec<i32> = string.split(",").map(|x| {
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

