#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate reqwest;
extern crate rand;
extern crate serde_json;

use serde::{ Deserialize, Serialize };
use serde_json::value::Value;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::{Json, JsonValue};
use rocket::http::{Status, ContentType};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::{thread, time};
use rand::Rng;

const GITHUB_API: &'static str = "https://api.github.com/";
const USER_AGENT: &'static str = "IssueMyst issue.myst.rs";
const MAX_ISSUES: u64 = 300;

/// runs the function, if function returns an error it returns an error
macro_rules! ret_on_error {
    ($e:expr) => {
        match $e {
            Ok(n) => n,
            Err(e) => return get_error_response(e)
        }
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct NIssues {
    open_issues: u64,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct User {
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Label {
    name: String,
    color: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RepoData {
    username: String,
    repo: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RateLimit {
    rate: Rate,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rate {
    limit: u64,
    remaining: u64,
    reset: u64
}

#[derive(Deserialize)]
#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

enum Error {
    CantReadPat,
    FailedToCreateRequest,
    InvalidGitHubResponse,
    NotFound,
    RateLimitReached,
    TooManyIssues,
    NoIssues,
}

struct ApiResponse {
    json: JsonValue,
    status: Status
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[post("/", data = "<repo>")]
fn post_random_issue(repo: Json<RepoData>) -> ApiResponse {
    let remaining = ret_on_error!(get_rate_limit_remaining());

    // if there is only 1 remaining in the rate limit return a server error
    if remaining <= 1 {
        return get_error_response(Error::RateLimitReached);
    }

    let repo_data = repo.into_inner();

    let n_issues = ret_on_error!(get_number_of_issues(&repo_data));

    if n_issues > MAX_ISSUES {
        return get_error_response(Error::TooManyIssues);
    }

    let mut issues = ret_on_error!(get_all_issues(&repo_data));

    // if no issues found return 404
    if issues.len() == 0 {
        return get_error_response(Error::NoIssues);
    }

    let rand_index = rand::thread_rng().gen_range(0, issues.len()-1);

    ApiResponse { json: JsonValue(serde_json::to_value(issues.remove(rand_index)).unwrap()), status: Status::Ok }
}

fn get_all_issues(repo: &RepoData) -> Result<Vec<Issue>, Error> {
    let mut page = 1;

    let mut issues: Vec<Issue> = Vec::new();

    // go through pages until the number of issues returned is 0
    loop {
        let url = format!("repos/{}/{}/issues?page={}", repo.username, repo.repo, page);
        let res = send_github_request(url)?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }

        match res.json::<Vec<Issue>>() {
            Ok(res) => {
                if res.len() == 0 {
                    return Ok(issues);
                }

                issues.extend(res);
            },
            Err(error) => {
                println!("{}", error);
                return Err(Error::InvalidGitHubResponse);
            }
        }

        page += 1;
        thread::sleep(time::Duration::from_millis(200))
    }
}

// gets the number of open issues a repo has
fn get_number_of_issues(repo: &RepoData) -> Result<u64, Error> {
    let url = format!("repos/{}/{}", repo.username, repo.repo);

    let res = send_github_request(url)?;

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::NotFound);
    }

    match res.json::<NIssues>() {
        Ok(res) => return Ok(res.open_issues),
        Err(error) => {
            println!("{}", error);
            return Err(Error::InvalidGitHubResponse);
        }
    };
}

/// gets the remaining amount of requests left before reaching the github rate limit
fn get_rate_limit_remaining() -> Result<u64, Error> {
    let res = send_github_request("rate_limit".to_string())?;

    match res.json::<RateLimit>() {
        Ok(r) => return Ok(r.rate.remaining),
        Err(error) => {
            println!("{}", error);
            return Err(Error::InvalidGitHubResponse);
        }
    };
}

/// sends a github api request
/// 
/// # Arguments
/// 
/// * endpoint - github api endpoint, gets appended to the github api url
fn send_github_request(endpoint: String) -> Result<reqwest::blocking::Response, Error> {
    let url = format!("{}{}", GITHUB_API, endpoint);

    let pat = get_pat()?;

    let client = reqwest::blocking::Client::new();
    let res = client.get(&url)
        .header("User-Agent", USER_AGENT)
        .bearer_auth(pat)
        .send();

    match res {
        Ok(res) => return Ok(res),
        Err(error) => {
            println!("{}", error);
            return Err(Error::FailedToCreateRequest);
        }
    };
}

/**
 * returns the personal access token from the pat.txt
 */
fn get_pat() -> Result<String, Error> {
    let file = File::open("pat.txt");

    let file = match file {
        Ok(f) => f,
        Err(error) => {
            println!("{}", error);
            return Err(Error::CantReadPat);
        }
    };

    let mut reader = BufReader::new(file);
    let mut contents = String::new();

    match reader.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            println!("{}", e);
            return Err(Error::CantReadPat);
        }
    };

    Ok(contents.trim().to_string())
}

fn get_error_response(error: Error) -> ApiResponse {
    let message = match error {
        Error::CantReadPat => "server failed to read the PAT".to_string(),
        Error::FailedToCreateRequest => "server failed to create a request".to_string(),
        Error::InvalidGitHubResponse => "got an invalid response from github".to_string(),
        Error::NotFound => "repo is either private or doesn't exist".to_string(),
        Error::NoIssues => "repo has no open issues".to_string(),
        Error::RateLimitReached => "the app has reached github's rate limit, try again in an hour".to_string(),
        Error::TooManyIssues => format!("the repo has more than {} issues, this is limited so the rate limit by github isn't reached so fast.", MAX_ISSUES)
    };

    let status = match error {
        Error::NotFound | Error::NoIssues => Status::NotFound,
        Error::RateLimitReached => Status::TooManyRequests,
        _ => Status::InternalServerError,
    };

    ApiResponse { json: JsonValue(Value::String(message)), status: status }
}

#[get("/")]
fn index() -> Template {
    Template::render("index", &())
}

fn main() {
    rocket::ignite().mount("/", routes![index, post_random_issue]).mount("/static", StaticFiles::from("static")).attach(Template::fairing()).launch();
}
