# clanker

[![Crates.io](https://img.shields.io/crates/v/clanker.svg)](https://crates.io/crates/clanker)

Clanker is a theme for fish with a focus on minimalism.

[![clanker demo](https://asciinema.org/a/274780.svg)](https://asciinema.org/a/274780)

## Installation

```sh
cargo install clanker
```

Then place this in your `config.fish` or somewhere that will be sourced when
`fish` starts up:

```fish
function fish_prompt
    set -g CLANKER_STATUS $status
    clanker-prompt
end

function fish_right_prompt
    clanker-right-prompt "$CLANKER_STATUS"
end

function fish_title
    clanker-title "$_"
end
```

### Building From Source

```sh
git clone git@github.com:Gregory-Meyer/clanker.git
cd clanker
cargo build --release
```

You will then need to copy the binaries from `target/release` to somewhere in
your `PATH`, like `/usr/local/bin`.

## Usage

### `clanker-prompt`

`clanker-prompt` outputs the current username, hostname, and compressed current
working directory. There are two optional arguments -- the unpriviliged and
priviliged line enders. These default to `'>'` and `'#'`, respectively, but you may
replace them at runtime using these arguments.

### `clanker-right-prompt`

`clanker-right-prompt` prints the status of the last command in red if it was
nonzero and some info the git repository the current folder is in. If the
current directory is a git repository according to `git_repository_open_ext(..., REPOSITORY_OPEN_FROM_ENV, ...)`, this program will print out some info about
`HEAD`. If `HEAD` points to a branch, the name of that branch will be printed.
If `HEAD` points to a tagged commit, the name of those tags will be printed. If
multiple tags point to the same commit as `HEAD`, then the tags are delimited
with a backslash (`'\'`). Otherwise, the shortened 7-digit SHA sum of the
current commit will be output.

### `clanker-title`

`clanker-title` optionally takes the currently running program as an argument
and prints it along with the compressed current working directory.

## Path Compression

Paths are compressed so that each compressed component is the shortest unique
prefix of a filename in that path. A component will never be shortened to `"."`
or `".."`. The last component in a path is never shortened. Components that do
not represent a unique prefix at all are not compressed. Home directories of
another user, like `~gregjm` or `~root`, are not compressed.


