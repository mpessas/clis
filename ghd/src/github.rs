use reqwest::blocking::{Client, Response};
use reqwest::Error;
use serde::Deserialize;
use url::{ParseError, Url};

const DEPLOYMENTS_BASE_URL: &str = "https://api.github.com/";

#[derive(Deserialize, Debug)]
struct DeploymentStatus {
    state: String,
}

impl DeploymentStatus {
    pub fn is_successful(&self) -> bool {
        return self.state == "success";
    }
}

#[derive(Deserialize, Debug)]
pub struct Deployment {
    sha: String,
    statuses_url: String,
}

impl<'a> Deployment {
    fn new(sha: String, statuses_url: String) -> Self {
        return Deployment {
            sha: sha,
            statuses_url: statuses_url,
        };
    }

    fn is_successful(&self, client: &'a ApiClient) -> bool {
        let url = Url::parse(self.statuses_url.as_str()).expect("Invalid deployment statuses URL");
        let statuses = client
            .make_get_request(&url)
            .expect("Request git Github deployment-statuses API failed")
            .error_for_status()
            .expect("Error fetching deployment statuses")
            .json::<Vec<DeploymentStatus>>()
            .expect("Error de-serializing deployment-statuses response");

        for ds in statuses.iter() {
            if ds.is_successful() {
                return true;
            }
        }
        return false;
    }
}

#[derive(Deserialize, Debug)]
pub struct GitCommit {
    pub message: String,
}

pub struct Repository<'a> {
    repo: String,
    client: &'a ApiClient,
}

impl<'a> Repository<'a> {
    pub fn new(repo: String, client: &'a ApiClient) -> Self {
        return Repository {
            repo: repo,
            client: client,
        };
    }

    pub fn last_successful_deployment(&self) -> Option<Deployment> {
        return match self
            .deployments()
            .expect("Error fetching deployments for repo")
            .iter()
            .find(|&d| d.is_successful(&self.client))
        {
            Some(d) => Some(Deployment::new(d.sha.clone(), d.statuses_url.clone())),
            None => None,
        };
    }

    pub fn commit_of_deployment(&self, deployment: &Deployment) -> Result<GitCommit, Error> {
        let url = self
            .client
            .build_url(
                &self.repo,
                format!("git/commits/{0}", deployment.sha).as_str(),
            )
            .expect("Error building the git-commit URL");
        return self
            .client
            .make_get_request(&url)
            .expect("Request to Github API failed")
            .error_for_status()
            .expect("Error getting commit object")
            .json::<GitCommit>();
    }

    fn deployments(&self) -> Result<Vec<Deployment>, Error> {
        let url = self
            .client
            .build_url(&self.repo, "deployments")
            .expect("Error building the deployments URL");
        let response = self
            .client
            .make_get_request(&url)
            .expect("Request to Github API failed");
        return response
            .error_for_status()
            .expect("Error getting deployments")
            .json::<Vec<Deployment>>();
    }
}

pub struct ApiClient {
    token: String,
}

impl ApiClient {
    pub fn new(token: String) -> Self {
        return ApiClient { token: token };
    }

    fn build_url(&self, repo: &str, part: &str) -> Result<Url, ParseError> {
        return Url::parse(DEPLOYMENTS_BASE_URL)?
            .join("repos/")?
            .join(format!("{repo}/").as_str())?
            .join(part);
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
}
