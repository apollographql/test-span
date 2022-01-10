<img src="https://raw.githubusercontent.com/apollographql/space-kit/main/src/illustrations/svgs/astronaut1.svg" width="100%" height="144">

# Contributing to test-span

## Before you contribute!

A general rule of thumbs is that if a contribution requires more than an 1 hour of work, chances are it's worth commenting on an issue and / or discussing it with the maintainers first.

That will allow us to figure out a way to solve the issue together, and possibly agree on what kind of PR would fix it best. Your time is valuable and we want to make sure you have the best contributors experience.

## Setting up the project

Test-span is written in [Rust]. In order to contribute, you'll need to have Rust installed. To install Rust,
visit [https://www.rust-lang.org/tools/install].

Rust has a build tool and package manager called [`cargo`] that you'll use to interact with test-span's code.

To build:

```bash
cargo build
```

To run tests:

```bash
cargo test
# If you would like to read the out put you can run
cargo test -- --nocapture
```

[apollo graphql]: https://www.apollographql.com
[rust]: https://www.rust-lang.org/
[`cargo`]: https://doc.rust-lang.org/cargo/index.html
[https://www.rust-lang.org/tools/install]: https://www.rust-lang.org/tools/install

## Pipelines

This project uses Circle CI to run a continuous integration and delivery pipeline. Every code change will be run against a few steps to help keep the project running at its peak ability.

- **CLA Check**: If you haven’t signed the Apollo CLA, a bot will comment on your PR asking you to do this
- **Tests**: The CI will run `cargo test`.
- **Lints**: The CI will check for lints and clippy compliance.

After you have opened your PR and all of the status checks are passing, please assign it to one of the maintainers (found in the bottom of [the README](./README.md#contributing) who will review it and give feedback.

# Code of Conduct

The project has a [Code of Conduct] that _all_ contributors are expected to follow. This code describes the _minimum_
behavior expectations for all contributors.

As a contributor, how you choose to act and interact towards your fellow contributors, as well as to the community, will
reflect back not only on yourself but on the project as a whole. The [Code of Conduct] is designed and intended, above all
else, to help establish a culture within the project that allows anyone and everyone who wants to contribute to feel
safe doing so.

Should any individual act in any way that is considered in violation of the
[Code of Conduct], corrective actions will be taken. It is possible, however, for any individual to _act_ in such a
manner that is not in violation of the strict letter of the [Code of Conduct] guidelines while still going completely
against the spirit of what that code is intended to accomplish.

Open, diverse, and inclusive communities live and die on the basis of trust. Contributors can disagree with one another
so long as they trust that those disagreements are in good faith and everyone is working towards a common goal.

## Bad Actors

All contributors to tacitly agree to abide by both the letter and spirit of the
[Code of Conduct]. Failure, or unwillingness, to do so will result in contributions being respectfully declined.

A _bad actor_ is someone who repeatedly violates the _spirit_ of the [Code of Conduct] through consistent failure to
self-regulate the way in which they interact with other contributors in the project. In doing so, bad actors alienate
other contributors, discourage collaboration, and generally reflect poorly on the project as a whole.

Being a bad actor may be intentional or unintentional. Typically, unintentional bad behavior can be easily corrected by
being quick to apologize and correct course _even if you are not entirely convinced you need to_. Giving other
contributors the benefit of the doubt and having a sincere willingness to admit that you _might_ be wrong is critical
for any successful open collaboration.

Don't be a bad actor.

[code of conduct]: https://github.com/apollographql/.github/blob/main/CODE_OF_CONDUCT.md

### Code review guidelines

It’s important that every piece of code in Apollo packages is reviewed by at least one core contributor familiar with that codebase. Here are some things we look for:

1. **Required CI checks pass.** This is a prerequisite for the review, and it is the PR author's responsibility. As long as the tests don’t pass, the PR won't get reviewed. To learn more about our CI pipeline, read about it [below](#pipelines)
2. **Simplicity.** Is this the simplest way to achieve the intended goal? If there are too many files, redundant functions, or complex lines of code, suggest a simpler way to do the same thing. In particular, avoid implementing an overly general solution when a simple, small, and pragmatic fix will do.
3. **Testing.** Please make sure that the tests ensure that the code won’t break when other stuff change around it. The error messages in the test should help identify what is broken exactly and how. The tests should test every edge case if possible. Please make sure you get as much coverage as possible.
4. **No unnecessary or unrelated changes.** PRs shouldn’t come with random formatting changes, especially in unrelated parts of the code. If there is some refactoring that needs to be done, it should be in a separate PR from a bug fix or feature, if possible.
5. **Code has appropriate comments.** Code should be commented describing the problem it is solving, not just the technical implementation. Avoid unnecessary comments if the code speaks well enough for itself.

<img src="https://raw.githubusercontent.com/apollographql/space-kit/main/src/illustrations/svgs/observatory.svg" width="100%" height="144">
