# aichangelog

![Crates.io](https://img.shields.io/crates/v/aichangelog)
![Crates.io](https://img.shields.io/crates/d/aichangelog)
![Crates.io](https://img.shields.io/crates/l/aichangelog)

[`aichangelog` is a CLI tool written in Rust](https://crates.io/crates/aichangelog), that generates a changelog based on your Git commit messages. It leverages OpenAI's conversational models to produce a human-readable, Markdown-formatted changelog.

## Installation

aichangelog can be easily installed with Cargo, Rust's package manager. Simply run the following command:

```bash
cargo install aichangelog
```

Please note that in order to use aichangelog, you will need to set the `OPENAI_API_KEY` environment variable. This API key is required to use the OpenAI language models, which is used by aichangelog to generate commit messages.

## Usage

### Generating Conventional Commits with `aichangelog`

<!-- START TABLE HERE -->
| Short | Long            | Description                                            | Default       |
| ----- | --------------- | ------------------------------------------------------ | ------------- |
| -s    | --short         | Only use first line of commit message to reduce tokens |               |
| -t    | --temp <TEMP>   | Temperature for AI 0.0 - 2.0                           | 1.0           |
| -f    | --freq <FREQ>   | Frequency Penalty for AI -2.0 - 2.0                    | 0.0           |
| -m    | --model <MODEL> | Model to use                                           | gpt-3.5-turbo |
| -h    | --help          | Print help                                             |               |
| -V    | --version       | Print version                                          |               |
<!-- END TABLE HERE -->


### Getting Help with `aichangelog`

To get help with using `aichangelog`, you can use the `-h` or `--help` option

```bash
$ aichangelog --help
```

This will display the help message with information on how to use the tool.