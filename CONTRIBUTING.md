# Contributing

Any contribution is welcome, feel free to open an issue or a pull request (add algorithm, improve performance,
fix issues, improve documentation, ...).

In case of pull request, please run [rustfmt](https://github.com/rust-lang/rustfmt) and
[clippy](https://github.com/rust-lang/rust-clippy) and fix any warning.

## Mendatory

As I use [release-plz](https://release-plz.dev/) to release new version, commit must follow
[conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) (and [release-plz details](https://release-plz.dev/docs/changelog/format)).

## Not mendatory but appreciated

If the pull request is about one phonetic algorithm, you can use the algorithm name as scope in
conventional commit.

It's also better if there is an issue for the pull request, especially for bugs since if people
find out bug they would most likely search if the issue is already reported (and I will probably
check for duplicates).

## Note on AI

I'm not fond of IA, and though I would probably won't refuse code, pull request, comments made by an AI,
I'd like that to keep it limited `:)` :

* code made by AI be limited (several lines is fine but large portion of code won't if I see it too much)
* comments, pull request descriptions are also ok but I would prefere if they are done by humans (but you
  can use it to translate from your native tongue to english, use AI as draft then wording on your own,
  improving wording).

And here [some rust-lang position](https://blog.rust-lang.org/inside-rust/2026/08/05/rust-langrust-is-adopting-an-llm-policy/)
I aggree with regarding AI.
