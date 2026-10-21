# TaxMan: type-safe bindings for HMRC's Making Tax Digital APIs

> Have you ever wanted to be able to do your taxes using a type-safe Rust API?

> No? What do you mean no?!

| API | crates.io | docs.rs |
|-----|-----------|---------|
| [Business Details](https://developer.service.hmrc.gov.uk/api-documentation/docs/api/service/business-details-api/) | [![Link](https://img.shields.io/crates/v/hmrc-mtd-business-details)](https://crates.io/crates/hmrc-mtd-business-details) | [![Link](https://img.shields.io/docsrs/hmrc-mtd-business-details)](https://docs.rs/hmrc-mtd-business-details) |
## TODOs
- [ ] Perhaps move in the openapi crate?
- [ ] Move the fraud prevention headers into the common crate.
- [ ] Investigate adding a Cargo pre-build hook to run the codegen.
- [.] This crate parses the OpenAPI specifications for HMRC's APIs and generates corresponding
types and functions to API schemas and endpoints.

- [ ] A CI pipeline triggers on a new commit to the official [HMRC changlelog repo](https://github.com/hmrc/income-tax-mtd-changelog), which will regenerate the code for the APIs, and file a
PR with the appropriate changes

- [ ] If an API changes from its last iteration (need a delta algorithm), auto-increment the patch version number if the HMRC-dictated version number has not changed.

- [ ] Automated testing: types are tested by deserializing the official example values given within the schemas themselves, and each endpoint is tested using the HMRC Sandbox environment.

- [ ] Upon merging, each API will be pushed to crates.io under separate `hmrc-mtd-*` crates, with shared types and utilities contained within the `hmrc-mtd` crate.

## Getting Started
### Installing

To use these bindings, include the desired APIs in your `Cargo.toml`:
```toml
[dependencies]
# Use the latest compatible version of common types.
hmrc-mtd = "*"
# Example of using pinned `major.minor` release for each API.
hmrc-mtd-vat = "1.0.*"
hmrc-mtd-business-details = "2.0.*"
```

Versions are typically updated fornightly on Fridays (in line with the HMRC changelog). To get the latest changes, you can run `cargo update`.

### Using the APIs

[![crates.io](https://shields.io)](https://crates.io)

## Contributing
