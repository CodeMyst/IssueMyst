#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate reqwest;
extern crate rand;

use serde::{ Deserialize, Serialize };
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use rocket::http::Status;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::{thread, time};
use rand::Rng;

const USER_AGENT: &'static str = "IssueMyst issue.myst.rs";

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Debug)]
struct Issue {
    url: String,
    html_url: String,
    labels_url: String,
    number: u64,
    title: String,
    user: User,
    labels: Vec<Label>,
    created_at: String,

}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Debug)]
struct User {
    login: String,
}

#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Debug)]
struct Label {
    name: String,
    color: String,
}

#[derive(Deserialize)]
#[derive(Serialize)]
struct RepoData {
    username: String,
    repo: String,
}

#[derive(Deserialize)]
#[derive(Serialize)]
struct RateLimit {
    rate: Rate,
}

#[derive(Deserialize)]
#[derive(Serialize)]
struct Rate {
    limit: u64,
    remaining: u64,
    reset: u64
}

#[post("/", data = "<repo>")]
fn post_random_issue(repo: Json<RepoData>) -> Result<Json<Issue>, Status> {
    let remaining;

    if let Ok(r) = get_rate_limit_remaining() {
        remaining = r;
    } else {
        return Err(Status::InternalServerError);
    }

    // if there is only 1 remaining in the rate limit return a server error
    if remaining <= 1 {
        return Err(Status::InternalServerError);
    }

    let mut issues;
    
    match get_all_issues(repo.into_inner()) {
        Ok(i) => issues = i,
        Err(e) => {
            if e == "404" {
                return Err(Status::NotFound);
            }
            return Err(Status::InternalServerError);
        }
    }

    // if no issues found return 404
    if issues.len() == 0 {
        return Err(Status::NotFound);
    }

    let rand_index = rand::thread_rng().gen_range(0, issues.len()-1);

    Ok(Json(issues.remove(rand_index)))
}

fn get_all_issues(repo: RepoData) -> Result<Vec<Issue>, String> {
    let mut page = 1;

    let client = reqwest::blocking::Client::new();

    let pat;

    if let Ok(p) = get_pat() {
        pat = p;
    } else {
        return Err("failed to get pat".to_string());
    }

    let mut issues: Vec<Issue> = Vec::new();

    // go through pages until the number of issues returned is 0
    loop {
        let url = format!("https://api.github.com/repos/{}/{}/issues?page={}", repo.username, repo.repo, page);
        let res = client.get(&url)
            .header("Authorization", format!("Bearer {}", pat))
            .header("User-Agent", USER_AGENT)
            .send();

        match res {
            Ok(res) => {
                if res.status() == 404 {
                    return Err("404".to_string());
                }

                if let Ok(res) = res.json::<Vec<Issue>>() {
                    if res.len() == 0
                    {
                        return Ok(issues);
                    }

                    issues.extend(res);
                } else {
                    return Err("failed to get response".to_string());
                }
            },

            Err(_) => {
                return Err("failed to send request".to_string());
            }
        }

        page += 1;
        thread::sleep(time::Duration::from_millis(200))
    }
}

fn get_rate_limit_remaining() -> Result<u64, String> {
    let pat;

    if let Ok(p) = get_pat() {
        pat = p;
    } else {
        return Err("failed to get pat".to_string());
    }

    let client = reqwest::blocking::Client::new();
    let res = client.get("https://api.github.com/rate_limit")
        .header("Authorization", format!("Bearer {}", pat))
        .header("User-Agent", USER_AGENT)
        .send();

    match res {
        Ok (res) => {
            if let Ok(res) = res.json::<RateLimit>() {
                return Ok(res.rate.remaining);
            } else {
                return Err("failed to get response".to_string());
            }
        },

        Err(_) => {
            return Err("failed to send request".to_string());
        }
    }
}

/**
 * returns the personal access token from the pat.txt
 */
fn get_pat() -> std::io::Result<String> {
    let file = File::open("pat.txt")?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(contents)
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &())
}

fn main() {
    rocket::ignite().mount("/", routes![index, post_random_issue]).mount("/static", StaticFiles::from("static")).attach(Template::fairing()).launch();
}
