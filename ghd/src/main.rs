use clap::Parser;

mod github;

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
    let gh_client = github::ApiClient::new(args.token.clone());

    let repo = github::Repository::new(args.repo.clone(), &gh_client);
    let deployment = match repo.last_successful_deployment() {
        Some(d) => d,
        None => panic!("No successful deployment"),
    };
    let commit = repo
        .commit_of_deployment(&deployment)
        .expect("Error de-serializing the git-commit object");
    println!("{}", commit.message);
}
