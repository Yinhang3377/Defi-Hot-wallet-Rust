Unresolved advisories (short-term mitigations)

- RUSTSEC-2023-0071 (rsa): no fixed upgrade available. Mitigation: limit uses of RSA, restrict keys, monitor usage, plan to replace or apply patch/fork.
- Any remaining ring instances were forced to 0.17.14 via patch.crates-io.

Action items:
- Track upstream patches / switch to maintained alternatives.
- Run `cargo audit` in CI for every PR.