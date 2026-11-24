# Changelog

Notable changes to `dioxus-floating` are documented here, newest first.

## [Unreleased]

## [0.3.3] — 2026-11-24

### Added

- Middleware playground comparing `Flip` and `Shift` in vertical and horizontal
  scroll boxes. The examples are built for WebAssembly in CI and deployed to
  [GitHub Pages](https://blarx.github.io/dioxus-floating/) from `main`.

### Changed

- Include both license texts for the declared `MIT OR Apache-2.0` licensing.

### Fixed

- Avoid a panic in `use_placement` and `use_placement_on_point` on native targets
  by using Tokio for the positioning delay. WebAssembly uses `gloo-timers`.
  ([#1](https://github.com/Blarx/dioxus-floating/pull/1))

### Compatibility

- On native targets, the positioning hooks require an active Tokio runtime with
  its time driver enabled.

## [0.3.2] — 2026-04-15

### Changed

- Updated the version in the README installation example.

## [0.3.1] — 2026-04-15

### Added

- `use_click_outside` calls a handler when the user clicks outside the element
  identified by a reactive ID.
- Standalone FlyonUI-styled `dropdown` and `context_menu` examples.

## [0.2.1] — 2026-01-29

### Added

- `OffsetOptions` supports independent horizontal and vertical offsets for
  floating elements.

### Compatibility

- `FloatingOptions::offset` now takes an `OffsetOptions` value in place of a
  single `f64`.

## [0.1.2] — 2026-01-25

### Added

- `ScrollableView` forwards global attributes and exposes handlers for mouse
  movement, button presses, and button releases.

### Changed

- `use_placement` and `use_placement_on_point` accept inputs convertible to
  `ReadSignal`, including signals and memos, and return a read-only
  `ReadSignal<FloatingResult>`.

## [0.1.1] — 2026-01-16

### Added

- `Placement` implements `PartialEq` for comparing placement values.

## [0.1.0] — 2026-01-14

### Added

- Initial release for Dioxus 0.7 with element- and point-based placement,
  `ScrollableView`, and `Flip` and `Shift` middleware.

[Unreleased]: https://github.com/Blarx/dioxus-floating/compare/v0.3.3...main
[0.3.3]: https://github.com/Blarx/dioxus-floating/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/Blarx/dioxus-floating/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/Blarx/dioxus-floating/compare/v0.2.1...v0.3.1
[0.2.1]: https://github.com/Blarx/dioxus-floating/compare/v0.1.2...v0.2.1
[0.1.2]: https://github.com/Blarx/dioxus-floating/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Blarx/dioxus-floating/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Blarx/dioxus-floating/tree/v0.1.0
