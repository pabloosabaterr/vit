```
      .__  __
___  _|__|/  |_
\  \/ /  \   __\
 \   /|  ||  |
  \_/ |__||__|
```
# Vit

Correlation search for git commits.

Vit builds a map of your repository's commit history where similar
commits end up near each other. Instead of grepping through messages,
you describe what you're looking for and Vit finds commits that mean
something similar, even with different words.

## How it works

Vit reads your commit messages, builds a TF-IDF matrix of word
co-occurrences, and extracts semantic dimensions via truncated SVD.
Words that appear in similar contexts get similar coordinates.
A commit's position is the weighted center of its words.

No external services, no API keys, no network requests. Everything
runs locally on your machine, also deterministic and no LLM behind.

## Usage

```
vit map              build the word map (run once, or after new commits)
vit near <message>   find commits closest to a message
vit help [command]   show help for a command
```

## Installation

```
cargo build --release
cp target/release/vit /usr/local/bin/
```

## Developing

```
cargo build --release
./target/release/vit <vit stuff>
```
> You do want to use the release build even while testing or developing or
> else it will take several x times more to run.

You can find docs about how to contribute and ideas of what at:

[CONTRIBUTING](./CONTRIBUTING.md)

You'll find also interesting:

[CODING-GUIDELINES](./CODING-GUIDELINES.md)

## Example

```
$ cd your-repo && vit map
  mapped 81294 commits, 12372 words, 111 dims

$ vit near "fix authentication bug"
  a1b2c3d   0.42  auth: fix token refresh on expired sessions
  d4e5f6a   0.58  login: handle invalid credentials gracefully
  b7c8d9e   0.71  auth: session cookie not set on redirect
```

## Name

"vit" comes from vectors + git, "vit" is Catalan for "seen"... also short
enough to type thousands of times without pissing me off.

I do like the name I promise :)

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT License](LICENSE-MIT), at your option.

---

Made with love and curiosity <3
