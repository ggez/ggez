# Contributing to ggez

Hi there! We're thrilled that you'd like to contribute to this project. Your help is essential for keeping it great.

## How to send in your contributions

There are many ways you can send your contributions to ggez. You can either **report a bug**, or you can make the changes yourself and **submit a pull request**!

### Reporting bugs and opening issues

Please [report bugs](https://github.com/ggez/ggez/issues) and open issues generously. Don't be afraid that your idea is silly, or you're reporting a duplicate. We're happy to hear from you. Seriously.

> ***Please Note:*** ggez is written by volunteers. If you encounter a problem while using it, we'll do our best to help you, but the authors cannot offer any support.

### Finding things to work on

Known bugs and feature requests are all in the [issue tracker](https://github.com/ggez/ggez/issues) so that's a good place to start looking for places to help.  Bugs marked `*GOOD FIRST ISSUE*` are fairly self-contained and probably don't need lots and lots of research. Others, especially those marked `*HARD*`, will often require a bit of finesse, or larger/broader changes to the library.

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

# Maintainer's Code of Conduct

Maintainers are the ones who accept or deny pull requests, make
releases, and generally choose long-term goals and designs.

The best thing I've ever seen for how to successfully run an open source
project has been the talk titled "Making Night In The Woods Better With
Open Source", at GDC 2017.  It is viewable here:
<https://www.youtube.com/watch?v=Qsiu-zzDYww> I've tried to do what it
says to do, and it seems to have worked pretty well.

Currently there is no real process for how maintainers are chosen,
besides an existing maintainer saying "hey, are you interested?".  These
communications should be done in public if possible, because it's a
public project.  Example: <https://github.com/ggez/ggez/issues/875>
Try to be a little conservative please, it's a lot easier to add
maintainers than remove them.

Here are rules for how to act when speaking for ggez as a maintainer.
People notice this project from time to time, and it has a reputation
for friendliness, ease of use that is worth almost as much as any
technical merit.  This tries to sum up how to maintain and carry forward
that reputation, so people keep doing cool stuff with the project.
Change the rules if you need to, but try to have good reasons for it.

As a maintainer, you promise:

 * I will be polite, even when noobs annoy me
 * I will not bash other projects, even when they do dumb stuff
 * I will uphold the Code of Conduct fairly and justly, even when I
   would rather hold a grudge
 * I will not abuse ggez or associated tools for personal gain, even
   when it would be really easy
 * I will try to prefer solutions that do 90% of the work with 10% of
   the code, even when it would be really fun to do the other 90%
 * I will keep the project about its core values: make good games,
   easily
