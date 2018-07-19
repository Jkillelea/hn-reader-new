use std::sync::{Arc, Mutex};
use std::io::Read;
use std::thread;

use url::Url;
use reqwest;
use serde_json;

pub const BASE_URL: &str = "https://hacker-news.firebaseio.com/";

#[derive(Debug)]
/// Possible returns from the ArticleBuffer iterator
pub enum ArticleResult {
    Text(String), // the actual article
    Waiting,      // waiting on IO,
}

/// ArticleBuffer struct holds a list of articles and allows a user to iterate over them
#[derive(Debug)]
pub struct ArticleBuffer {
    buffer: Arc<Mutex<Vec<String>>>,
    thread_running: Arc<Mutex<bool>>,
}

impl ArticleBuffer {
    /// Create an ArticleBuffer and spawn a background thread to fill it
    pub fn new(ids: Vec<i32>) -> ArticleBuffer {
        let article_buffer = ArticleBuffer {
            buffer: Arc::new(Mutex::new(Vec::new())),
            thread_running: Arc::new(Mutex::new(true))
        };

        let buf     = article_buffer.buffer.clone(); // cloning these only clones a *handle*
        let running = article_buffer.thread_running.clone();
        thread::spawn(move || { // Launch a new thread
            let base   = Url::parse(BASE_URL).unwrap();
            let client = reqwest::Client::new();
            for id in ids { // for each article ID
                let mut body = String::new();
                let url = base.join( // Generate a URL
                                    &format!("/v0/item/{}.json?print=pretty", id)
                                ).unwrap();
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
    type Item = ArticleResult; // The type of item we'll return is a string

    // The only required method. Each time it is called, it should either return
    // the next item that we want, or None.
    fn next(&mut self) -> Option<Self::Item> {
        let article = self.buffer.lock().unwrap().pop();
        if article.is_some() {                    // See if an article is ready
            Some(ArticleResult::Text(article.unwrap())) // and if so, return it
        } else {
            if *self.thread_running.lock().unwrap() { // If we're waiting on network
                return Some(ArticleResult::Waiting);
            }
            None // Actually out of content? Return None
        }
    }
}