# `my-iot`

## `master`

- New: generic persisted JSON key-value store
- Break: add `expires_ts` to the key-value store
- Break: rename `Measurement` into `Reading`
- New: add `is_persistent` flag to `Reading`
- New: add `/sensors/{sensor}/json` endpoint
- Break: services must spawn threads themselves

## `0.7.0`

- Chore: refactor service initialisation & running
- New: add favicons
- Break: introduce service IDs
- New: set thread names
- New: status page

## `0.6.0`

- New: add individual sensor page
