# BTRFS Dedupe

This is a BTRFS deduplication utility. It operates in a batch mode, scanning
for files with the same size, performing an SHA256 hash on each one, then
invoking the kernel deduplication ioctl for all those that match.

It is written by [James Pharaoh](james@pharaoh.uk).

It is hosted at [gitlab.wellbehavedsoftware.com]
(https://gitlab.wellbehavedsoftware.com/well-behaved-software/wbs-backup).

It is also available from the following locations:

* [crates.io](https://crates.io/crates/btrfs-dedupe)

* [Github]
(https://github.com/wellbehavedsoftware/wbs-backup/tree/master/btrfs-dedupe) —
this is a clone of the gitlab repository, where bug reports etc should be made.

* [WBS Dist](https://dist.wellbehavedsoftware.com/btrfs-dedupe/) — this contains
binary packages for Ubuntu trusty and xenial.

## Alternatives

There are two alternatives, of which I am aware:

* [Duperemove](https://github.com/markfasheh/duperemove) — Performs a
block-level hash on files and attempts to deduplicate parts of files. This is
overkill for my purposes, although I have no reason to believe it does not work
well. I believe it will be slower than this tool, since it does a far deeper
analysis of file contents.

* [Bedup](https://github.com/g2p/bedup) — Performs a similar task to this tool,
plus it keeps a database of files in order to avoid checksumming again. The main
implementation, however, does not use the kernel ioctls (which were simply not
available when it was created), although a branch supports this. It also suffers
from leaving filesystems in an inconsistent state in the case of errors, namely
setting files as immutable, and it also crashes if there are many files to
deduplicate.

## General information

The utility is very simple. It takes a list of directories, scans for files with
matching sizes, performs an SHA256 checksum on each one, then invokes the ioctl
to deduplicate the entire file for every match it finds. Optionally, it can
match filenames as well as sizes; this may make the program run faster in some
cases.

## Usage

From the built-in help:

```
$ btrfs-dedupe --help

Btrfs Dedupe 

USAGE:
    btrfs-dedupe [FLAGS] [<PATH>]

FLAGS:
    -h, --help              Prints help information
        --match-filename    Match filename as well as checksum
    -V, --version           Prints version information

ARGS:
    <PATH>...    Root path to scan for files
```
