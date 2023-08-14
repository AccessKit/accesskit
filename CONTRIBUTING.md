# Contributing to AccessKit

## Making a Pull Request

### Cargo.lock File

AccessKit intentionally includes the `Cargo.lock` file in our git repository.
This is mainly due to our bindings to other languages.

Usually you should not run cargo update when creating a pull request even when adding/changing a dependency.
Simply building the library should update the `Cargo.lock` file with the minimal changes needed.

> [!NOTE]
> This is not normal / best practice for most libraries.
> See the [official documentation](https://doc.rust-lang.org/cargo/faq.html#why-do-binaries-have-cargolock-in-version-control-but-not-libraries) for more information. 

### CHANGELOG.md

Our CHANGELOG.md files are auto generated using [Release Please](https://github.com/googleapis/release-please) and should not be edited manually.

To control what is in the CHANGELOG.md for your change, use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) when writing your commit messages.

#### Example Commit Messages
Taken from [Conventional Commits documentation](https://www.conventionalcommits.org/en/v1.0.0/#summary):
> feat: allow provided config object to extend other configs
>
> BREAKING CHANGE: `extends` key in config file is now used for extending other config files

> fix: prevent racing of requests
>
> Introduce a request id and a reference to latest request. Dismiss
incoming responses other than from latest request.

### Testing Locally

We have some platform specific tests that are not run with `cargo test` from the project root by default.

1. Run test that work on all platforms
   ``` shell
   cargo test
   ```
2. Run platform specific tests.

   Run the appropriate command for your platform
   ``` shell
   cargo test -p accesskit_windows
   cargo test -p accesskit_unix
   cargo test -p accesskit_macos
   ```
   Not all platforms have tests at this time, but they may in the future.

> [!WARNING]
> **Windows**: Some tests may fail if the created window looses focus while testing. This can also happen when using the terminal built into your IDE. Try running them from your native terminal if you get failures. 
