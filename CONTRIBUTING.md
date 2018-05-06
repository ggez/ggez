# Contributing to ggez

Hi there! We're thrilled that you'd like to contribute to this project. Your help is essential for keeping it great.

## How to send in your contributions

There are many ways you can send your contributions to ggez. You can either **report a bug**, or you can make the changes yourself and **submit a pull request**!

### Reporting bugs and opening issues

Please [report bugs](https://github.com/ggez/ggez/issues) and open issues generously. Don't be afraid that your idea is silly, or you're reporting a duplicate. We're happy to hear from you. Seriously.

> ***Please Note:*** ggez is written by volunteers. If you encounter a problem while using it, we'll do our best to help you, but the authors cannot offer any support.

### Finding things to work on

Known bugs and feature requests are all in the [issue tracker](https://github.com/ggez/ggez/issues) so that's a good place to start looking for places to help.  Bugs marked `*EASY*` are fairly self-contained and probably don't need lots and lots of research.  Bugs marked `*LESS EASY*` will require a bit of finesse, or larger/broader changes to the library.

### Submitting a pull request

* [Fork](https://github.com/ggez/ggez/fork) and clone the repository
* Create a new branch: git checkout -b my-branch-name
* Make your changes
> Ideally all commits will contain no use of `unwrap()`, no compiler warnings and all tests will pass.
> It's advised to run _latest_ [rustfmt](https://github.com/rust-lang-nursery/rustfmt) and [clippy](https://github.com/rust-lang-nursery/rust-clippy) before submitting a pull request
* Push to your fork and [submit a pull request](https://github.com/ggez/ggez/compare) to the `devel` branch
* Pat your self on the back and wait for your pull request to be reviewed.

If you're unfamiliar with how pull requests work, [GitHub's documentation on them](https://help.github.com/articles/using-pull-requests/) is very good.

Here are a few things you can do that will increase the likelihood of your pull request being accepted:

* Update the documentation as necessary, as well as making code changes.
* Keep your change as focused as possible. If there are multiple changes you would like to make that are not dependent upon each other, consider submitting them as separate pull requests.
* [Write a good commit message](http://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html).

### Branches

All of ggez's in-progress work happens on the `devel` branch.  The `master` branch tracks the current latest release.  When we make
a major release, we merge the `devel` branch into `master`, and from then on only backwards-compatible changes get merged from
`devel` into `master`.

For example, when we release `0.3.0`, we create a new branch for `0.2` from `master`, `devel` gets merged into `master` and the
release gets made from `master`.  If we then discover and fix a bug in `devel`, we can merge the changes fixing that bug into the
`master` branch, and make a `0.3.1` release from it.

### Code and other contributions

Contributions to ggez (via pull request or otherwise) must be licensed under the same license as ggez

### Submitting examples

The purpose of the example code is to be documentation of ggez's features.  Unfortunately, examples are also a maintenance burden, so we
don't want to just include every cool little program we write.  Examples that just use features that already are shown off by other examples should be
kept to a minimum... though this doesn't mean we can't refactor several example programs into one, or vice versa, or that there has to be no
duplication at all.

If you've written something cool and want to show it off, but it doesn't fulfill the listed guidelines, consider making it its own project
and submitting a PR to add it to the `docs/Projects.md` file!
