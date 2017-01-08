# BTRFS Dedupe

This is a BTRFS deduplication utility. It operates in a batch mode, scanning
for files with the same size, performing an SHA256 hash on each one, then
invoking the kernel deduplication ioctl for all those that match.

It is written by [James Pharaoh](james@pharaoh.uk).

It is released into the public domain under the permissive [MIT license]
(https://opensource.org/licenses/MIT). Dependencies may have other licenses,
please be aware that these may apply to statically linked binary releases.

It is hosted at [btrfs-dedupe.com]
(http://btrfs-dedupe.com) — please report any issues or feature requests here.

It is also available from the following locations:

* [WBS Gitlab](https://github.com/wellbehavedsoftware/btrfs-dedupe)

* [Crates.io](https://crates.io/crates/btrfs-dedupe)

* [Github]
(https://github.com/wellbehavedsoftware/wbs-backup/tree/master/btrfs-dedupe) —
this is a clone of the gitlab repository, where bug reports etc should be made.
Pull requests are welcome here, but issues should be reported [here]
(https://gitlab.wellbehavedsoftware.com/well-behaved-software/wbs-backup/issues).

* [WBS Dist](https://dist.wellbehavedsoftware.com/btrfs-dedupe/) — this contains
binary packages for Ubuntu trusty and xenial.

## General information

The current version of this utility is designed for batch operation, and it uses
a state file to enable successive executions to operate incrementally. It will
first scan the file system and create an index of all files present, it then
takes an SHA256 checksum for each file, then it takes an SHA256 checksum of a
representation of the file extent map for each file. Finally, for every set of
two or more files with a matching content hash but different extent hashes, it
will execute the defragment ioctl for the first, then the deduplicate ioctl
against this file for every other.

It saves its state regularly to a file which is simply a list of JSON entries,
one for each file present, along with some metadata (size, mtime, etc), the
content hash, the extent hash, and the timestamps for taking each hash and for
performing deduplication. This file is gzipped to save space, and probably time
as well.

It will automatically skip content hashes for files which don't appear to have
changed (from the metadata), it will skip extent hashes for files which don't
appear to have changed (from the content hash), and it will skip deduplication
for files which already appear to be deduplicated (from the extent hash and
deduplication timestamp).

This tool can take multiple paths, and can operate on a subset of the filesystem
comprising the sum of these parts. It will maintain its database if it is run
successively with different parts of the filesystem, only considering the
specified paths to operate on, and then work correctly if run over a wider or
different selection of paths at a later time.

I believe this will work on other file systems which support these standard
IOCTLs, but I have not tested this. In particular, I believe XFS should work. I
have not tested this; please let me know any success or failure if you attempt
this.

## Warning

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
I also have a tool which complements this, [RZBackup](http://rzbackup.com/).

I also offer commercial backup solutions, with very competitive pricing. Please
contact [sales@wellbehavedsoftware.com](mailto:sales@wellbehavedsoftware.com)
for more information.

## Usage

From the built-in help `btrfs-dedupe dedupe --help`:

```
USAGE:
    btrfs-dedupe dedupe [OPTIONS] [<PATH>]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --content-hash-batch-size <SIZE>
            Amount of file contents data to hash before writing database
            [default: 2GiB]
        --database <PATH>
            Database path to store metadata and hashes
        --dedupe-batch-size <SIZE>
            Amount of file data to deduplicate before writing database
            [default: 64GiB]
        --extent-hash-batch-size <SIZE>
            Amount of file extent data to hash before writing database
            [default: 512GiB]
        --minimum-file-size <SIZE>
            Minimum file size to consider for deduplication [default: 1KiB]

ARGS:
    <PATH>...    Root path to scan for files

```

In general, you need to choose a location for your database, for example
`/var/cache/btrfs-dedupe/database.gz`, and make sure this directory exists. I'm
assuming you are going to run as root.

```sh
mkdir /var/cache/btrfs-dedupe
```

Then you can run the dedupe process on a regular basis. It's a good idea to do
so before you make any read-only snapshots. For example, I make snapshots
nightly, and run the dedupe process beforehand to ensure that my snapshots don't
contain duplicated data.

```sh
btrfs-dedupe dedupe --database /var/cache/btrfs-dedupe/database.gz /btrfs
```

You can add as many paths as you like, but btrfs-dedupe assumes that all the
paths you provide are on the same btrfs filesystem. If not, then it's probably
not going to work very well.

## Roadmap

The following features are planned:

* Option to include/exclude files according to patterns.

* Option to force update of stored data on a regular basis, for a subset of
  files which are selected in a periodic way (eg each file gets a forced recheck
  once every 'n' days, which can be configured).

* Options to control defragmentation options, or to turn it off, and to enable
  defragmentation for directories.

Please let me know if you are keen to see any of these features, or if there is
anything else you would like to see in btrfs-dedupe.

## Alternatives

There are various alternatives, documented on the BTRFS wiki:

https://btrfs.wiki.kernel.org/index.php/Deduplication

## FAQ

### Deduplication of read only snapshots

It is not currently possible to deduplicate read-only snapshots, except perhaps
to deduplicate an extent in a read-write subvolume from one in a read-only
snapshot.

It is possible to create a read-write snapshot from a read-only one, perform the
deduplication, and then create a new read-only snapshot. This could be done
automatically and I may create a script to automate this. However, this does
change the snapshot's internal "identity" in a way that will break some things,
for example the send/receive functionality which relies on these identities.

It is recommended to run deduplication _before_ you create snapshots, and on a
longer term basis snapshots should probably be archived in a different manner,
for example using ZBackup (which is mentioned above), which provides its own
very efficient deduplication and compression.
