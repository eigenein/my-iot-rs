# `my-iot`

## `master`

- Break: update thread closures to return `Result` #32
- Break: run a supervised thread in a loop #32

## `0.24.0`

- Opt: use `crossbeam-channel` instead of `bus` #28 #33

## `0.23.0`

- Break: drastically change service spawn interface #18
- Break: remake actions
- New: initial Telegram service #18
- New: send sensor update and change messages
- New: scenario conditions and actions are optional
- Fix: make `disabled_services` setting optional
- Opt: remove useless database cache

## `0.21.0`

- Break: use TOML instead of YAML for configuration
- New: initial Telegram support #18
- New: `Condition::Or` is able to accept any number of child conditions
- Opt: speed up database with cache #23
- Chore: `cargo update`
- Chore: rename `crate::reading` into `crate::message`

## `0.20.0`

- Chore: add `crossorigin="anonymous"` to Font Awesome script
- Chore: update docs and add redirect to `my_iot`
- Opt: vacuum database on startup #14

## `0.19.5`

- Chore: move docs to Wiki

## `0.19.4`

- Chore: `cargo update`

## `0.19.3`

- Fix: bump `eventsource` to `0.4.0`
- Chore: `cargo update`

## `0.19.2`

- Fix: rollback to `eventsource` from crates.io

## `0.19.1`

- Fix: bump `eventsource` to fix build on Raspberry Pi

## `0.19.0`

- Opt: remove `openssl` dependency
- Chore: `cargo update`

## `0.18.0`

- Chore: bump Bulma to 0.7.5
- Chore: `cargo update`
- Fix: danger zone layout

## `0.17.0`

- Opt: use `bus` with `std::sync::mpsc` instead of `multiqueue` to reduce CPU usage

## `0.16.2`

- Fix: thread names

## `0.16.1`

- Opt: switch back to `multiqueue`
- Chore: use module path plus service ID as thread name

## `0.16.0`

- New: add danger zone #9, not functional yet
- Opt: add `loading="lazy"` to the `ImageUrl` renderer
- Break: services will exchange messages instead of readings
- Opt: use `multiqueue2`

## `0.15.0`

- New: Nest camera snapshot URL
- Break: change Nest sensors to include device type
- Chore: get rid of `humansize` crate
- Fix: render inline and non-inline items differently

## `0.14.0`

- Break: replace `crossbeam-channel` with `multiqueue` for the broadcasting
- New: more automator conditions

## `0.13.0`

- New: `human_format` for human-readable value formatting
- New: `Automator` service, unusable yet
- Chore: basic unit test for `Buienradar`
- Break: change `:` sensor separator to `::`

## `0.12.0`

- Chore: remove some `unwrap()`s
- New: `--settings` command-line option
- New: `--db` command-line option

## `0.11.0`

- Chore: prepare for publishing

## `0.10.0`

- Chore: learn to not use `unwrap()`
- Chore: implement unimplemented value rendering

## `0.9.0`

- Break: key-value store will store different data types in their own tables
- Break: remove status page, it is not yet clear what it should contain
- Break: use `crossbeam_channel` for message passing so that other services could listen to readings too
- Break: use service IDs as sensor prefixes
- Chore: make use of `serde` default attribute
- New: dark theme via `bulma-prefers-dark`
- New: Nest API service and a few initial sensors

## `0.8.0`

- Break: add `expires_ts` to the key-value store
- Break: rename `Measurement` into `Reading`
- Break: services must spawn threads themselves
- New: add `/sensors/{sensor}/json` endpoint
- New: add `is_persistent` flag to `Reading`
- New: generic persisted JSON key-value store

## `0.7.0`

- Break: introduce service IDs
- Chore: refactor service initialisation & running
- New: add favicons
- New: set thread names
- New: status page

## `0.6.0`

- New: add individual sensor page
