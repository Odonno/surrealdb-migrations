# Contributing <!-- omit in toc -->

Thank you for investing your time in contributing to this project! Any contribution you make will be reflected on the [README](readme.md) :sparkles:.

In this guide you will get an overview of the contribution workflow from opening an issue, creating a PR, reviewing, and merging the PR.

## Issues

### Create a new issue

If you spot a problem in this project, [search if an issue already exists](https://docs.github.com/en/github/searching-for-information-on-github/searching-on-github/searching-issues-and-pull-requests#search-by-the-title-body-or-comments). If a related issue doesn't exist, you can open a new issue using a relevant [issue form](https://github.com/Odonno/surrealdb-migrations/issues/new).

### Solve an issue

Scan through our [existing issues](https://github.com/Odonno/surrealdb-migrations/issues) to find one that interests you. You can narrow down the search using `labels` as filters. See [Labels](/contributing/how-to-use-labels.md) for more information. If you find an issue to work on, you are welcome to open a PR with a fix.

#### New contributors

If you are new to this project and you desire to start contributing, you can check the [list of issues for newcomers](https://github.com/Odonno/surrealdb-migrations/issues?q=is%3Aopen+is%3Aissue+label%3A"good+first+issue").

#### Help wanted

If you want to provide your help, here is a [list of issues where you can contribute](https://github.com/Odonno/surrealdb-migrations/issues?q=is%3Aopen+is%3Aissue+label%3A"help+wanted").

## Coding standards

This project is built using:

- [Visual Studio Code](https://code.visualstudio.com/)
- The [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) vscode extension
- That should run `cargo fix` and `cargo clippy` automatically

## Make Changes

### Getting started from source

1. Install rust

To set up a working **development environment**, you will need the latest `rust` version.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install surrealdb cli

If you want to work on the latest release, you will need the `nightly` version:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh -s -- --nightly
```

If you want to apply a fix for the current stable version, you will need to install the stable version:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh
```

3. Fork the project

```bash
git clone git@github.com:[YOUR_FORK_HERE]/surrealdb-migrations.git
cd surrealdb-migrations
```

### Testing

Unit testing is an important part of the software development lifecycle. Writing unit tests is not mandatory, but it is always a nice addition. If possible, please try to write unit tests on relatively small functions.

Unit testing is important but this project requires a lot of integration/end-to-end testing to ensure the reliability of the CLI and the Rust crate. In order to write integration tests, you will need to run 2 local SurrealDB instances locally with the following specifications:

- a local SurrealDB instance accessible on port `8000` with a root user having username `root` and password `root`
- a second local SurrealDB instance accessible on port `8001` with a root user having username `admin` and password `admin`

To avoid mistakes and remove the unnecessary steps to set up the requirements for the tests, there is a bash file inside the project that you can run before starting the tests:

```
./before-integration-tests.sh
```

## Definition of Done

You should always review your own PR first. For content changes, make sure that you:

- [ ] Confirm that the changes meet the user experience and goals outlined in the content design plan (if there is one).
- [ ] Compare your pull request's source changes to staging to confirm that the output matches the source and that everything is rendering as expected. This helps spot issues like typos, content that doesn't follow the style guide, or content that isn't rendering due to versioning problems. Remember that lists and tables can be tricky.
- [ ] Review the content for technical accuracy.
- [ ] If there are any failing checks in your PR, troubleshoot them until they're all passing.
- [ ] Make sure you wrote the necessary unit and integration tests that cover your changes in order to prevent future regressions.
- [ ] Update the project docs and [README](readme.md) to prevent outdated documentation.

## Commits

Every commit should follow these rules:

- Every commit should be atomic meaning that it should cover only **one** specific feature, scenario or bugfix. Following this logic, commits should not be too small or repetitive. If multiple commits treat of the same subject, consider squashing them together.
- Every commit should follow the [Gitmoji](https://gitmoji.dev/) commit convention. To keep things simple, there are no mechanism to enforce this rule like pre-commit hooks.

### Commit convention

Here is a link to the detailed Gitmoji specifications: [https://gitmoji.dev/specification](https://gitmoji.dev/specification)

#### Valid commit

✅ Here is an example of a valid commit:

```bash
✨ add cli arg to create schemafull table/event
```

#### Invalid commits

❌ Unicode emoji should be preferred over shortcode emoji

```bash
:sparkles: add cli arg to create schemafull table/event
```

❌ There should always be a single whitespace between emoji and message

```bash
✨add cli arg to create schemafull table/event
```

❌ The first letter should not be uppercase

```bash
✨ Add cli arg to create schemafull table/event
```

❌ The message should start with an action verb

```bash
✨ cli arg to create schemafull table/event
```

❌ Ideally, the message should not be too long but it should clearly better describe what the commit is about (for performance improvements: what is being improved?)

```bash
⚡️ improve performances
```

## Pull Request

When you're finished with the changes, create a pull request, also known as a PR.

- Don't forget to [link PR to issue](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue) if you are solving one.
- We may ask for changes to be made before a PR can be merged, either using [suggested changes](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/incorporating-feedback-in-your-pull-request) or pull request comments. You can apply suggested changes directly through the UI. You can make any other changes in your fork, then commit them to your branch.
- As you update your PR and apply changes, mark each conversation as [resolved](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/commenting-on-a-pull-request#resolving-conversations).
- If you run into any merge issues, checkout this [git tutorial](https://github.com/skills/resolve-merge-conflicts) to help you resolve merge conflicts and other issues.

## Your PR is merged!

Congratulations :tada::tada:

Once your PR is merged, your contributions will be publicly visible on the contributors list of the [README](https://github.com/Odonno/surrealdb-migrations#lets-see-paul-allens-contributions).
