//! CLI tool to check all non-archived repositories in the Research Square Company organization for
//! appropriate repository topics.

use anyhow::Result;
use graphql_client::*;
use list_repositories::ListRepositoriesSearchNodesOnRepository;
use std::env;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.json",
    query_path = "graphql/list-repositories.graphql",
    response_dervices = "Debug"
)]
pub struct ListRepositories;

/// Convenience structure to store a list of repositories along with the next pagination cursor
struct RepositoryResponse {
    repositories: Vec<ListRepositoriesSearchNodesOnRepository>,
    after_cursor: Option<String>,
}

/// Retrieve a batch of non-archived repositories starting after the provided cursor
fn get_repositories(
    client: &reqwest::blocking::Client,
    after_cursor: Option<String>,
) -> Result<RepositoryResponse> {
    let query = ListRepositories::build_query(list_repositories::Variables {
        query: "org:researchsquare archived:false".to_string(),
        after: after_cursor,
    });

    let res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(env::var("GITHUB_TOKEN")?)
        .json(&query)
        .send()?;

    res.error_for_status_ref()?;

    let response_body: Response<list_repositories::ResponseData> = res.json()?;
    let response_data = response_body.data.expect("missing response data");

    let page_info = response_data.search.page_info;
    let repositories: Vec<ListRepositoriesSearchNodesOnRepository> = response_data
        .search
        .nodes
        .expect("No nodes found")
        .into_iter()
        .filter_map(|node| node)
        .filter_map(|node| match node {
            list_repositories::ListRepositoriesSearchNodes::Repository(repository) => {
                Some(repository)
            }
            _ => None,
        })
        .collect();

    Ok(RepositoryResponse {
        repositories,
        after_cursor: page_info.end_cursor,
    })
}

/// Filter method to determine if the repository has one of the approved team topics configured
fn missing_required_topic(repository: &ListRepositoriesSearchNodesOnRepository) -> bool {
    let team_topics = vec![
        "platform",
        "delivery",
        "customer",
        "automation",
        "reporting",
        "operations",
    ];

    let topics: Vec<&str> = repository
        .repository_topics
        .nodes
        .as_ref()
        .expect("No repository topic found")
        .iter()
        .filter_map(|node| node.as_ref())
        .map(|node| node.topic.name.as_ref())
        .collect();

    !topics.iter().any(|topic| team_topics.contains(topic))
}

fn main() -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;

    let mut after_cursor = None;
    loop {
        let repository_response = get_repositories(&client, after_cursor)?;
        after_cursor = repository_response.after_cursor;

        repository_response
            .repositories
            .into_iter()
            .filter(missing_required_topic)
            .for_each(|repository| println!("{}", repository.name));

        if after_cursor.is_none() {
            break;
        }
    }

    Ok(())
}
