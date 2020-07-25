# `0.89.0`

- ✨ `SimpleAnomalyDetector`, closes #125

# `0.88.0`

- ✨ New `--suppress-log-timestamps` CLI option, closes #111
- 💥 Replace the `http_port` setting with the `http.port` setting

# `0.87.0`

- ⬆️ Upgrade dependencies
- ✨ New `sensor_pattern` setting for `Rhai` service to allow the easy filtering
- ✨ New `Threshold` service #115

# `0.86.0`

- 💥 YouLess: replace the `url` setting with the `host` setting

# `0.85.0`

- 🚸 Add coloring for `Value::BatteryLife`
- 💥 `Value::WindDirection` is dropped in favour of `Value::StringEnum`
- 🚸 Notify when there is no data points for a chart
- ♻️ Use `TryInto` to convert values for a chart
- ♻️ Introduce `TryInto` conversions for `Value` and use them in `rhai`
- ♻️ Refactor `Value` conversions
- 🚸 Notify when there is no chart for a sensor
- 🚸 Add navigation link to the book
- ⬆️ Upgrade dependencies

# `0.84.0`

- ✨ Add a short alias for the `--service-id` option
- ♻️ Use `matches!` macro where applicable
- ✨ New `is_online` sensor for Ring doorbells
- 🚸 Make Home and Settings navbar items visible on mobile devices

# `0.83.0`

- ⚡️ Remove unnecessary OpenSSL dependency
- 🗑 Remove unnecessary `engine.register_type` calls
- 🔇 Suppress `sending on a disconnected channel`, closes #112
- 🗑 Remove the publish caching because it doesn't work anyway

# `0.82.6`

- ⚡️ Use `slim` image for Raspberry Pi builds #117

# `0.82.5`

- ⚗ Fix the caching #117

# `0.82.3`

- ⚗ Checking the caching of the Docker images #117

# `0.82.2`

- ⚗ Fix the caching #117

# `0.82.1`

- ⚗ Checking the caching of the Docker images #117

# `0.82.0`

- ⚗ Use BuiltKit caching to speed up the publish action #117
- ⚗ Cross-building for `armv7-unknown-linux-gnueabihf`

# `0.81.1`

- 🥅 Do not fail the entire recordings loop when failed to download a one Ring recording
- ⚗ Increase the `CLIENT` timeout to mitigate api.ring.com slowness

# `0.81.0`

- ⚡️ Avoid unnecessary cloning of binary large objects
- ✨ Add `options` parameter to Telegram in Rhai #93 #18
- 💥 Removed content type from `Value::Blob`. I'll use `tree_magic` later, if needed

# `0.80.0`

Let's try some [gitmoji](https://gitmoji.carloscuesta.me/) in the changelog!

- ✨ Implement Rhai `telegram.send_video` function #93 #18
- ♻️ Refactor `Telegram` calls

# `0.79.0`

- Ring minimum viable service #61
- Changed `Telegram.send_message` interface

# `0.78.2`

- Remove the service worker cache because it doesn't play well with basic authentication

# `0.78.1`

- Raise an error for any unsuccessful response in the service worker

# `0.78.0`

- Remove `migrations` for a while – I want to redesign it
- Add user data database API
- Refactor the service `spawn` functions
- Designing the Ring.com service #61

# `0.77.1`

- Fix `credentials: same-origin` for PWA

# `0.77.0`

- Drop `Lua` service #93

# `0.76.0`

- Change `Telegram` Rhai interface and refactor the `Rhai` code #93

# `0.75.1`

- `spawn_process` logging #93

# `0.75.0`

- [Rhai](https://schungx.github.io/rhai/) scripting, closes #93

# `0.74.0`

- Fix `index` hero title
- [Rhai](https://schungx.github.io/rhai/) scripting (in progress) #93

# `0.73.3`

- Enable `Counter` and `DataSize` for `f64` charts

# `0.73.2`

- Fix `human_format` for the strict zeroes

# `0.73.1`

- Fix rain sensors for OpenWeather, rename sensors

# `0.73.0`

- OpenWeather API service, closes #16 #110

# `0.72.0`

- Sensor auto-refresh period to depend on a selected chart period #105
- Remove the Raw data section

# `0.71.1`

- Room title is now obligatory and defaults to `"Home"`

# `0.71.0`

- Cache only 2xx responses, closes #104
- Set caching headers on statics, closes #103
- Drop sensor TTLs, closes #102
- Display sensor reading count
- Delete sensors from the UI, closes #9
- Update `askama`

# `0.70.3`

- Fix [overflow on mobile](https://github.com/jgthms/bulma/issues/2769) #51

# `0.70.2`

- Fix `ETag` comparison #2

# `0.70.1`

- Fix `ETag` for NGINX #2

# `0.70.0`

- Implement `ETag` for the sensor view #2

# `0.60.2`

- Fix PWA display style

# `0.60.1`

- Fix `crossOrigin` on the PWA manifest

# `0.60.0`

- PWA, closes #51

# `0.59.5`

- Refactor `Value` rendering
- Fix kWh-to-joules conversion and display energy in Wh

# `0.59.4`

- Improve the chart title

# `0.59.3`

- Remove `make install` and `make uninstall` targets
- Decrease tado° refresh period to 3 minutes

# `0.59.2`

- Add `make tag` and `make tag/publish` targets

# `0.59.1`

- Improve `Power` colors and fix `Temperature` colors

# `0.59.0`

- Sensor side panel
- Fix `None` icon

# `0.58.0`

- Upsert messages in bulks #97

# `0.57.0`

- Fix `::open_window_activated` message type
- Add sensor JSON link to the sensor page, closes #94
- Add side navigation panel, closes #96

# `0.56.0`

- Fix 500 on the Settings page
- Print message count in the footer, closes #52

# `0.55.1`

- Remove extra calls in Tado° service #73

# `0.55.0`

- Fix sensor titles
- Refactor Tado° service #73
- Add `--service-id` CLI option to run only specified services
- Replace rustdoc with mdBook
- Emulate Tado° Open Window Detection Skill #73

# `0.54.0`

- Wrap `upsert_into` into a single transaction #97

# `0.53.0`

- Increase YouLess `default_interval_millis`

# `0.52.0`

- Added more Tado° sensors

# `0.51.0`

- Added the sensor TTL options

# `0.50.0`

- **Removed the `uom` and `rmp-serde` dependencies and changed the serialization format, thus dropped sensors, readings and migrations (requires to re-create the database)**
- Added YouLess service #53
- Added more Tado° sensors
- Bunch of refactorings
- Bundled Font Awesome, closes #95
- Initial charts #88

# `0.49.0`

- Introduced an individual sensor time-to-live, closes #86
- Add the first sensor to Tado° #73

# `0.48.0`

- Renamed Buienradar `wind_speed_bft` to `wind_force`
- Lower `default_max_sensor_age_ms` to 14 days
- Added ambient temperature to the dashboard
- Got rid of `supervisor`, thus `*::is_running` sensors have disappeared

# `0.47.0`

- Settings view page

# `0.46.0`

- `my-iot` now accepts multiple settings files. They're concatenated and the result is parsed as if it was a single TOML file.
- `Telegram` and `Solar` secrets have been moved into the separate `secrets` section, it means that from now on you can split secrets and non-secrets between different TOML files.

# `0.45.0`

Service intercommunication via "fake" messages doesn't play that nice. However, it was mainly meant to work with the old `Automator`, which is now replaced with `Lua`. Thus, I remove this functionality and leave `Write` messages only to actually change existing sensor value. Fake sensors become discouraged since now.

`Telegram` service won't listen to `Write` messages anymore. Use service methods from Lua instead.

**All sensors and readings will be dropped again.** Sensor primary and foreign keys have been changed to [SeaHash](https://ticki.github.io/blog/seahash-explained/)-ed sensor IDs. This allowed removing the dependency on auto-generated primary keys.

Settings file is now specified as an optional positional argument: `my-iot my-iot.toml`.

**`Nest` service has been removed due to the Google migration.**

Switched to [Rocket](https://rocket.rs/).

# `0.44.0`

Unified sensor IDs by changing `my-iot::*` to `my_iot::*`.

`Lua` service gets the new settings, `filter_sensor_ids` and `skip_sensor_ids`, to simplify filtering (out) messages that will be passed to `onMessage`.

# `0.43.0`

This release removes the old `Automator` in favour of the new `Lua` service.

Also, I'm removing the automatic `*::update` and `*::change` events, because they require additional database queries to be executed on each event. If needed, similar functionality can be implemented in Lua. See, for example, the recipe to send new Nest camera animations to Telegram.

`Service` trait is re-introduced to simplify service instantiation.

New `Solar` service is introduced to emit durations before and after sunset and sunrise.

# `0.42.0`

- Bundle most of the statics, closes #77
- Remove `disabled_services` from settings
- Lua automation #59

# `0.41.0`

- New sensors for `Db`
- Spawn `Db` service by default
- Bunch of refactorings
- Introduce `ConnectionExtensions` instead of standalone functions
- Denormalize `sensors` to speed up selecting actual values from a large database on low-performance boards

# `0.40.0`

- Add initial room title support
- Different sizes for dashboard cards
- Bump Bulma, bump Askama, fix navbar active flag
- Performance tricks, closes #72
- Bump Rust version
- New: sensor titles #10
- Fix: latest reading formatting
- Chore: improve database migrations

# `0.39.2`

- Chore: order dashboard by `sensor_id`
- Fix: only one migration gets applied on a startup (critical)

# `0.39.1`

- Fix: use `FULL` synchronous on SQLite

# `0.39.0`

- **Break: use `rmp-serde` and `uom` for readings persistence, the migration will delete all readings #69**
- New: database migrations #55

# `0.38.2`

- Opt: upgrade `reqwest`, use `rustls` #70

# `0.38.1`

- Chore: upgrade Rust #70

# `0.38.0`

- Fix: upgrade to OpenSSL `1.1.1d`

# `0.37.0`

- Opt: refactor message bus #58
- Opt: refactor `main`
- Opt: split `services` into `core` and service-dependent parts
- Chore: improve `Dockerfile` #64
- Break: remove unimplemented danger zone on the sensor page

# `0.36.0`

- Fix: sensors got recreated for each new reading (critical)

# `0.35.0`

- Chore: do not mark release as a prerelease anymore to allow getting the latest release via GitHub API #64

# `0.34.0`

- Fix: GitHub Actions

# `0.33.0`

- Fix: GitHub Actions

# `0.32.0`

- Fix: GitHub Actions

# `0.31.0`

- Fix: GitHub Actions

# `0.30.0`

- New: publish releases on GitHub

# `0.29.0`

This release brings the different database schema, which should work faster and take less disk space.
**The new schema is backwards-incompatible.**

- Break: change sensor value serialization #50
- Break: re-structure modules #50
- Break: change database schema #50
- New: cross-compilation for Raspberry Pi Zero (W) #62

# `0.28.0`

- Chore: Telegram producer thread returns `Result`

# `0.27.0`

- Break: switch to structopt #41
- New: logging level option #40

# `0.26.0`

- New: sunrise and sunset fields for Buienradar
- Fix: docs link
- Chore: simplify Buienradar date/time deserialization #36
- Chore: switch to `simple_logger` #42

# `0.25.0`

- Break: update thread closures to return `Result` #34
- Break: run a supervised thread in a loop #34
- New: send `my-iot::start` message on startup
- New: thread `is_running` sensor #34
- New: Status page link (no page itself yet)

# `0.24.0`

- Opt: use `crossbeam-channel` instead of `bus` #28 #33

# `0.23.0`

- Break: drastically change service spawn interface #18
- Break: remake actions
- New: initial Telegram service #18
- New: send sensor update and change messages
- New: scenario conditions and actions are optional
- Fix: make `disabled_services` setting optional
- Opt: remove useless database cache

# `0.21.0`

- Break: use TOML instead of YAML for configuration
- New: initial Telegram support #18
- New: `Condition::Or` is able to accept any number of child conditions
- Opt: speed up database with cache #23
- Chore: `cargo update`
- Chore: rename `crate::reading` into `crate::message`

# `0.20.0`

- Chore: add `crossorigin="anonymous"` to Font Awesome script
- Chore: update docs and add redirect to `my_iot`
- Opt: vacuum database on startup #14

# `0.19.5`

- Chore: move docs to Wiki

# `0.19.4`

- Chore: `cargo update`

# `0.19.3`

- Fix: bump `eventsource` to `0.4.0`
- Chore: `cargo update`

# `0.19.2`

- Fix: rollback to `eventsource` from crates.io

# `0.19.1`

- Fix: bump `eventsource` to fix build on Raspberry Pi

# `0.19.0`

- Opt: remove `openssl` dependency
- Chore: `cargo update`

# `0.18.0`

- Chore: bump Bulma to `0.7.5`
- Chore: `cargo update`
- Fix: danger zone layout

# `0.17.0`

- Opt: use `bus` with `std::sync::mpsc` instead of `multiqueue` to reduce CPU usage

# `0.16.2`

- Fix: thread names

# `0.16.1`

- Opt: switch back to `multiqueue`
- Chore: use module path plus service ID as thread name

# `0.16.0`

- New: add danger zone #9, not functional yet
- Opt: add `loading="lazy"` to the `ImageUrl` renderer
- Break: services will exchange messages instead of readings
- Opt: use `multiqueue2`

# `0.15.0`

- New: Nest camera snapshot URL
- Break: change Nest sensors to include device type
- Chore: get rid of `humansize` crate
- Fix: render inline and non-inline items differently

# `0.14.0`

- Break: replace `crossbeam-channel` with `multiqueue` for the broadcasting
- New: more automator conditions

# `0.13.0`

- New: `human_format` for human-readable value formatting
- New: `Automator` service, unusable yet
- Chore: basic unit test for `Buienradar`
- Break: change `:` sensor separator to `::`

# `0.12.0`

- Chore: remove some `unwrap()`s
- New: `--settings` command-line option
- New: `--db` command-line option

# `0.11.0`

- Chore: prepare for publishing

# `0.10.0`

- Chore: learn to not use `unwrap()`
- Chore: implement unimplemented value rendering

# `0.9.0`

- Break: key-value store will store different data types in their own tables
- Break: remove status page, it is not yet clear what it should contain
- Break: use `crossbeam_channel` for message passing so that other services could listen to readings too
- Break: use service IDs as sensor prefixes
- Chore: make use of `serde` default attribute
- New: dark theme via `bulma-prefers-dark`
- New: Nest API service and a few initial sensors

# `0.8.0`

- Break: add `expires_ts` to the key-value store
- Break: rename `Measurement` into `Reading`
- Break: services must spawn threads themselves
- New: add `/sensors/{sensor}/json` endpoint
- New: add `is_persistent` flag to `Reading`
- New: generic persisted JSON key-value store

# `0.7.0`

- Break: introduce service IDs
- Chore: refactor service initialisation & running
- New: add favicons
- New: set thread names
- New: status page

# `0.6.0`

- New: add individual sensor page
