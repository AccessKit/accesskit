# Contributing to AccessKit

## Reporting an issue

When reporting an issue, in order to help the maintainers understand what the problem is, please make your description of the issue as detailed as possible:

- if it is a bug, please provide clear explanation of what happens, what should happen, and how to reproduce the issue, ideally by providing a minimal program exhibiting the problem
- if it is a feature request, please provide a clear argumentation about why you believe this feature should be supported by AccessKit

## Making a Pull Request

When making a code contribution to AccessKit, before opening your pull request please make sure that:

- your patch builds with AccessKit's minimal supported Rust version (currently Rust 1.70)
- you added tests where applicable
- you tested your modifications on all impacted platforms (see below)
- you updated any relevant documentation
- you left comments in your code explaining any part that is not straightforward, so that the maintainers and future contributors don't have to guess what your code is supposed to do

### Cargo.lock File

AccessKit intentionally includes the `Cargo.lock` file in the git repository.

You should not run `cargo update` when creating a pull request even when adding or changing a dependency.
Simply building the library will update the `Cargo.lock` file with the minimal changes needed.
Remember to commit these changes as part of your pull request.

### CHANGELOG.md

Our `CHANGELOG.md` files are auto generated using [Release Please](https://github.com/googleapis/release-please) and should not be edited manually.

To control how your work will be described in the changelog, use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) when writing the title of your pull request.
If you think one line is not enough, mention it in your pull request so that maintainers can update the description of the merge commit.

### Testing Locally

We have platform-specific tests that do not run when doing `cargo test` from the project root directory.

1. Run cross-platform tests:
   ``` shell
   cargo test
   ```
2. Run platform-specific tests by issuing the appropriate command for your platform:
   ``` shell
   cargo test -p accesskit_macos
   cargo test -p accesskit_unix
   cargo test -p accesskit_windows
   ```

> [!WARNING]
> **Windows**: Some end-to-end tests may fail if the created window loses focus. This can happen when using the terminal built into your IDE. Try running them from Powershell or the Command Prompt instead.
