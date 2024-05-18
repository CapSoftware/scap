# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.4](https://github.com/helmerapp/scap/compare/scap-v0.0.3...scap-v0.0.4) - 2024-05-18

### Added
- allow excluding windows while recording
- add frame type BGRA for fastest mac capture

### Fixed
- intel machine fix applied to local fork of core-graphics
- use fork of core-graphics
- pass shows cursor option to stream config

### Other
- format frame funcs
- Merge pull request [#59](https://github.com/helmerapp/scap/pull/59) from helmerapp/fix/allow-building-for-intel-mac
- use forked apple-sys with latest version of bindgen
- bump up screencapture-kit version to 0.2.8
- Merge pull request [#53](https://github.com/helmerapp/scap/pull/53) from helmerapp/feat/add-support-for-shows-cursor
