<div align="center">
<h1>Crane</h1>
<a href="https://wakatime.com/badge/github/TheRustyPickle/Crane"><img src="https://wakatime.com/badge/github/TheRustyPickle/Crane.svg" alt="wakatime"></a>
</div>

Crane is a simple GUI application built with [Iced](https://github.com/iced-rs/iced) for managing binaries installed with cargo install. It lets you check for updates, reinstall, or remove installed crates through a minimal interface inspired by [pamac-manager](https://github.com/manjaro/pamac).

## Features

* View all installed Cargo binaries
* Lock a crate to prevent updates
* Enable or disable default and optional features
* Install or update crates directly from a git source (--git flag)
* Remove installed crates
* View real-time installation logs

https://github.com/user-attachments/assets/96b9758a-01bc-41cc-bb17-23e1ee5d2ded

## Motivation

I use a number of tools installed through cargo install (like `diesel_cli`, `cargo-nextest`, `cargo-binstall`, and `trunk`) and there’s no simple way to see when updates are available. Crane provides a small, focused interface for that.

## Installation

The app is not available on crates.io right now. Binary release is coming soon.

```
git clone https://github.com/TheRustyPickle/Crane
cd Crane
cargo run --release
```

## License

Crane is under the [MIT License](LICENSE).
