
use futures::TryStreamExt;
use std::error::Error;
use std::{convert::Infallible};
use std::process::Command;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

mod config;
mod record;
use record::*;
use config::Config;

struct BackupCmd<'a> {
    port: u16,
    context: &'a str,
    local: &'a str,
    destination: &'a str,
}

impl <'a>BackupCmd<'a> {
    pub(crate) fn new() -> Result<Self, Box<dyn Error>> {
        unimplemented!();
    }
    pub(crate) fn run(&self) -> Result<String, Box<dyn Error>> {
        #[cfg(target_os = "linux")]
        match String::from_utf8(Command::new("scp")
            .args([&format!("-P{}", self.port), self.local, self.destination])
            .output()?
            .stdout) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e))
        }
        #[cfg(not(target_os = "linux"))]
        // the windows version of scp requires a lowercase p while the linux version does require
        // it
        match String::from_utf8(Command::new("scp")
            .args([&format!("-p{}", self.port), self.local, self.destination])
            .output()?
            .stdout) {
            Ok(v) => Ok(v),
            Err(e) => Err(Box::new(e))
        }
    }
    fn check_room() -> bool {
        true
    }
}

#[tokio::main]
async fn main() {
    let upload = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(10_000))
        .and_then(upload);
    // add host
    // remove host
    let list = warp::path("list")
        .and_then(list_files);

    tokio::spawn(async move {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(3)).unwrap();
        watcher.watch("./config.json", RecursiveMode::Recursive).unwrap();
        while let Ok(v) = rx.recv() {
            match v {
                notify::DebouncedEvent::Write(v) => {},
                _ => continue
            }
        }
    });

    let router = upload.or(list).recover(handle_rejection);
    println!("Server started at 0.0.0.0:8080");
    warp::serve(router).run(([0, 0, 0, 0], 1234)).await;
}

async fn list_files() -> Result<impl Reply, Rejection> {
    let mut list = Vec::new();
    for file in std::fs::read_dir(".").unwrap() {
        list.push(file.unwrap().file_name().into_string().unwrap());
    }
    Ok(list.join("\n"))
}

async fn upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        if p.name() == "file" {
            let content_type = p.content_type();
            let file_ending;
            match content_type {
                Some(file_type) => match file_type {
                    "application/pdf" => {
                        file_ending = "pdf";
                    }
                    "image/png" => {
                        file_ending = "png";
                    }
                    v => {
                        eprintln!("invalid file type found: {}", v);
                        return Err(warp::reject::reject());
                    }
                },
                None => {
                    eprintln!("file type could not be determined");
                    return Err(warp::reject::reject());
                }
            }

            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    bytes::BufMut::put(&mut vec, data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            // let file_name = format!("./files/{}.{}", Uuid::new_v4().to_string(), file_ending);
            // tokio::fs::write(&file_name, value).await.map_err(|e| {
            //     eprint!("error writing file: {}", e);
            //     warp::reject::reject()
            // })?;
            // println!("created file: {}", file_name);
        }
    }
    Ok("Success")
}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid Requedst Header".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
