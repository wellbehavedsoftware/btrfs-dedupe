# BTRFS Dedupe

This is a BTRFS deduplication utility. It operates in a batch mode, scanning
for files with the same size, performing an SHA256 hash on each one, then
invoking the kernel deduplication ioctl for all those that match.

It is written by [James Pharaoh](james@pharaoh.uk).

It is released into the public domain under the permissive [MIT license]
(https://opensource.org/licenses/MIT). Dependencies may have other licenses,
please be aware that these apply to statically linked binary releases.

It is hosted at [btrfs-dedupe.com]
(http://btrfs-dedupe.com) — please report any issues or feature requests here.

It is also available from the following locations:

* [Crates.io](https://crates.io/crates/btrfs-dedupe)

* [Github]
(https://github.com/wellbehavedsoftware/wbs-backup/tree/master/btrfs-dedupe) —
this is a clone of the gitlab repository, where bug reports etc should be made.
Pull requests are welcome here, but issues should be reported [here]
(https://gitlab.wellbehavedsoftware.com/well-behaved-software/wbs-backup/issues).

* [WBS Dist](https://dist.wellbehavedsoftware.com/btrfs-dedupe/) — this contains
binary packages for Ubuntu trusty and xenial.

## General information

The utility is very simple. It takes a list of directories, scans for files with
matching sizes, performs an SHA256 checksum on each one, then invokes the ioctl
to deduplicate the entire file for every match it finds. Optionally, it can
match filenames as well as sizes; this may make the program run faster in some
cases.

*IMPORTANT CAVEAT* &mdash; I have read that there are race and/or error
conditions which can cause filesystem corruption in the kernel implementation of
the deduplication ioctl. I have also been told that this is not the case in the
newest kernels, and can't find the original comment, so hopefully this is not an
issue.

I have personally experienced many "corrupted" BTRFS filesystems but have in
almost every case been able to recover the data. The only exception to this was,
I believe, caused by corruption of the underlying block device and, of course, I
was able to detect the issue due to the integrity verification code in BTRFS and
recover the file in question from my backup.

If you don't have backups, I would recommend [ZBackup](http://zbackup.org/), and
I also have a tool which complements this, [RZBackup]
(https://gitlab.wellbehavedsoftware.com/well-behaved-software/wbs-backup/tree/master/btrfs-dedupe).

I also offer commercial backup solutions, with very competitive pricing. Please
contact [sales@wellbehavedsoftware.com](mailto:sales@wellbehavedsoftware.com)
for more information.

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

## Alternatives

There are two alternatives, of which I am aware:

* [Duperemove](https://github.com/markfasheh/duperemove) — Flexible tool which
is capable of block- and file-level deduplication, highly configurable, and
supports a database of previous mtimes and checksums to improve speed. This tool
has improved substantially since I last looked into it, and so my information
about it is incomplete.

* [Bedup](https://github.com/g2p/bedup) — Performs a similar task to this tool,
plus it keeps a database of files in order to avoid checksumming again. The main
implementation, however, does not use the kernel ioctls (which were simply not
available when it was created), although a branch supports this. It also suffers
from leaving filesystems in an inconsistent state in the case of errors, namely
setting files as immutable, and it also crashes if there are many files to
deduplicate.

There is also [ongoing work]
(http://www.mail-archive.com/linux-btrfs%40vger.kernel.org/msg32862.html) to
enable automatic realtime deduplication in the filesystem itself, but this is
likely to take a long time to stablise, and there are fundamental issues with
the concept which make it unsuitable for many cases.

There is a [wiki page](https://btrfs.wiki.kernel.org/index.php/Deduplication)
with general information about the state of deduplication in BTRFS.

## Roadmap

The following features are planned:

* Maintain a database of file checksums and modification times, similar to
bedup, in order to avoid checksumming files which have not (apparently) changed.

* Use BTRFS metadata to identify changed files, by comparing to snapshots or
perhaps some other internal data, to enable files which have the same mtime, but
which have changed, to be rescanned even if they are in the database.

* Use BTRFS metadata to identify if files are already deduplicated, and avoid
invoking the ioctl. I'm not sure if this is done automatically, but the speed at
which a repeat invocation runs makes me think that it is not.

* Extra options to limit scans to a single filesystem, and to include/exclude
files according to patterns.
