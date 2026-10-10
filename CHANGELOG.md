# Changelog

## [0.2.1](https://github.com/Project-Colony/Collector/compare/v0.2.0...v0.2.1) (2026-10-09)


### Fixes

* leave loopback traffic out of the network rates and list disks by mount point ([#9](https://github.com/Project-Colony/Collector/issues/9)) ([a901d28](https://github.com/Project-Colony/Collector/commit/a901d28e59c547543d355d1a584186bab5abe354))
* stop crashing when a reading goes over 100% and restore the terminal on SIGTERM and SIGHUP ([#7](https://github.com/Project-Colony/Collector/issues/7)) ([0c304a6](https://github.com/Project-Colony/Collector/commit/0c304a6e178fd8299544a06df1b0b10325fe0bb6))


### Internals

* split the program into cli, terminal, app and ui modules ([#11](https://github.com/Project-Colony/Collector/issues/11)) ([e14fc3c](https://github.com/Project-Colony/Collector/commit/e14fc3c0a2752d248c3470e34ee06a93c78a1c35))

## [0.2.0](https://github.com/Project-Colony/Collector/releases/tag/v0.2.0) (2026-10-08)


### Features

* add --help, --version and Ctrl+C, restore the terminal on panic, and ship signed releases ([359363f](https://github.com/Project-Colony/Collector/commit/359363fef60d3c3c69d16858fb8f4288d5ce8059))


### Fixes

* refresh the readings once per second, not on every key event ([#2](https://github.com/Project-Colony/Collector/issues/2)) ([76aff8a](https://github.com/Project-Colony/Collector/commit/76aff8a344cf14ce4cc4fc238be3bd541118e7ef))
