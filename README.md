# clanker

[![Crates.io](https://img.shields.io/crates/v/clanker.svg)](https://crates.io/crates/clanker)

Clanker is a minimalistic set of command prompts for fish.

## Usage

First, install the package via `cargo install clanker`.

Then put this in your `config.fish` or somewhere that will be sourced, ensuring
that the location `cargo` installs to is in your `PATH`:

```fish
function fish_prompt
    clanker-prompt
end

function fish_right_prompt
    clanker-right-prompt "$status"
end

function fish_title
    clanker-title "$_"
end
```

### `clanker-prompt`

`clanker-prompt` takes no arguments and outputs the current username, hostname,
and compressed directory. Compression of the current working directory is as
expected - first, if in the home directory, it's substituted with `~`. If there
is more than one component in the path, all components but the last are
trimmed to one extended grapheme cluster, or two if the first is a `.`.

### `clanker-right-prompt`

`clanker-right-prompt` optionally takes the status of the last command as an
argument. If the status was provided and was nonzero, it is printed in red. In
addition, if the current directory is a git repository according to the
behavior when running `git_repository_open_ext` with
`REPOSITORY_OPEN_FROM_ENV`, the current branch or checkout out commit ID will
be printed. If there are changes that would show up in `git status`, this will
be indicated with a `*`.

### `clanker-title`

`clanker-title` optionally takes the currently running program as an argument
and prints it along with the compressed current working directory.
