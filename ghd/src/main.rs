use clap::Parser;
use reqwest::blocking::{Client, Response};
use reqwest::Error;
use serde::Deserialize;
use url::{ParseError, Url};

const DEPLOYMENTS_BASE_URL: &str = "https://api.github.com/";

#[derive(Deserialize, Debug)]
struct Deployment {
    sha: String,
}

#[derive(Deserialize, Debug)]
struct GitCommit {
    message: String,
}

struct GithubApi {
    token: String,
}

impl GithubApi {
    pub fn get_commit_message_for_latest_deployment(&self, repo: &str) -> String {
        let url_deployments = self.build_deployments_url(repo).expect("Invalid URL");
        let response = self
            .make_get_request(&url_deployments)
            .expect("Request to Github API failed");
        let deployments = response
            .error_for_status()
            .expect("Error from Github")
            .json::<Vec<Deployment>>()
            .expect("Error de-serializing the response");

        let url_commit = self
            .build_commit_object_url(repo, deployments[0].sha.as_str())
            .expect("Invalid URL");
        let commit = self
            .make_get_request(&url_commit)
            .expect("Request to Github API")
            .error_for_status()
            .expect("Error getting commit object")
            .json::<GitCommit>()
            .expect("Error de-serializing the response");
        return commit.message;
    }

    fn make_get_request(&self, url: &Url) -> Result<Response, Error> {
        let client = Client::new();
        let req = client
            .get(url.clone())
            .header("Accept", "application/vnd.github+json")
            .header("X-Github-API-Version", "2022-11-28")
            .header("User-Agent", "ghd")
            .bearer_auth(self.token.clone())
            .query(&[("environment", "prod")]);
        return req.send();
    }

    fn build_deployments_url(&self, repo: &str) -> Result<Url, ParseError> {
        return self.build_base_api_url(repo)?.join("deployments");
    }

    fn build_commit_object_url(&self, repo: &str, sha: &str) -> Result<Url, ParseError> {
        return self
            .build_base_api_url(repo)?
            .join(format!("git/commits/{sha}").as_str());
    }

    fn build_base_api_url(&self, repo: &str) -> Result<Url, ParseError> {
        return Url::parse(DEPLOYMENTS_BASE_URL)?
            .join("repos/")?
            .join(format!("{repo}/").as_str());
    }
}

/// Get the latest deployment of a repository
#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]
struct Args {
    /// The repository (owner/repo)
    repo: String,

    /// The github token to use
    #[arg(short, long)]
    token: String,
}

fn main() {
    let args = Args::parse();
    let gh_api = GithubApi { token: args.token };
    println!(
        "{}",
        gh_api.get_commit_message_for_latest_deployment(args.repo.as_str())
    );
}
