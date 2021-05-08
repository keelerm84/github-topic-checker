# github-topic-checker

Research Square Company requires that all repositories have a topic set which
represents team ownership. This is useful for reporting purposes. This small
CLI app will check each non-archived repository in that organized and report on
repositories lacking one of the required topics.

## Environment variables

To run this project, you will need to have the following environment variables set.

* `GITHUB_TOKEN`

   This [personal access token][tokens] is used to create deployments on any target repositories.

## Setup and usage

Install [cargo-make][cargo-make] and generate an updated copy of the GitHub
GraphQL schema.json file.

```console
$ cargo install cargo-make
$ cargo make schema
```

Compile and execute this application

```console
$ cargo build --release
$ ./target/release/github-topic-checker
```

## License

[MIT](./LICENSE.md)

[tokens]: https://github.com/settings/tokens
[cargo-make]: https://github.com/sagiegurari/cargo-make
