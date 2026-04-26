# Project Structure

This file is the high-level map for the repository so contributors can find the main logic without walking every folder manually.

## Folder Structure

```text
.
|-- src/
|   `-- lib.rs
|-- stellar-insured-contracts/
|   |-- common/
|   |-- contracts/
|   |-- docs/
|   |-- oracle/
|   |-- scripts/
|   |-- sdk/
|   |-- security-audit/
|   |-- security-tests/
|   `-- tests/
|-- CHANGELOG.md
`-- README.md
```

## Logic Map

To find the Solidity governance event bridge visit [src/lib.rs](src/lib.rs).

To find the Rust and ink! contract workspace visit [stellar-insured-contracts/README.md](stellar-insured-contracts/README.md).

To find contract implementations visit [stellar-insured-contracts/contracts/README.md](stellar-insured-contracts/contracts/README.md).

To find contributor-facing architecture and integration notes visit [stellar-insured-contracts/docs/README.md](stellar-insured-contracts/docs/README.md).

To find SDK integration surfaces visit [stellar-insured-contracts/sdk/README.md](stellar-insured-contracts/sdk/README.md).

To find integration and benchmark test entry points visit [stellar-insured-contracts/tests/README.md](stellar-insured-contracts/tests/README.md).

The security test suite can be found in [stellar-insured-contracts/security-tests/README.md](stellar-insured-contracts/security-tests/README.md).

The mobile SDK guide can be found in [stellar-insured-contracts/sdk/mobile/README.md](stellar-insured-contracts/sdk/mobile/README.md).

## Architectural Decisions

The top-level Solidity file stays separate from the Rust workspace because it models a small governance execution/event concern, while the Rust workspace contains the larger PropChain and Stellar Insured contract system.

The `stellar-insured-contracts/contracts` directory is the main production contract layer. The `sdk`, `tests`, and `docs` directories support that layer instead of defining primary on-chain behavior.

## Tradeoffs

Focused folder documentation is used instead of README files in every small contract subdirectory. That keeps navigation useful while avoiding documentation churn in a large open-source repository.

Source files are linked with relative paths so GitHub and local clones both resolve the references.
