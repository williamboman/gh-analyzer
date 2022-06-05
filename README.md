# gh-analyzer

GitHub's insights capabilities are pretty limited, and will only give you data
for a limited time range (past 2 weeks). This (very simple) tool uses GitHub's
API to fetch and persist insights data, allowing you to better understand
long-term usage patterns and trends of your repositories - blazingly fast.

## Install

```sh
$ cargo install --git https://github.com/williamboman/gh-analyzer
```

## Requirements

- [`gh` cli](https://cli.github.com/)

## Example usage

```
0 0 * * * gh-analyzer --out-dir /srv/github-stats repo    williamboman/gh-analyzer
0 0 * * * gh-analyzer --out-dir /srv/github-stats clones  williamboman/gh-analyzer
0 0 * * * gh-analyzer --out-dir /srv/github-stats traffic williamboman/gh-analyzer
```

## Notes

Historic data have a tendency to be volatile, where historic numbers provided
by the API seem to change over time. This is worth keeping in mind when
ingesting the data into another system, like a time series database.
