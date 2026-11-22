# Known-good registry fixture

`irig106-types-0.1.0.crate` is the actual published archive, frozen in Git for
the read-only publish-path check. It is not rebuilt or fetched as an expected
value by CI. Do not replace it when adopting a new release version.

- Download: https://crates.io/api/v1/crates/irig106-types/0.1.0/download
- Metadata: https://crates.io/api/v1/crates/irig106-types/0.1.0
- Recorded 2026-11-22 SHA-256:
  `2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420`.
- The metadata's registry checksum matches the fixture. Its
  `.cargo_vcs_info.json` matches `trustpub_data.sha`:
  `3c517358694646652939c236968b202ebf76e28a`, publishing run `43727921675`,
  successful attempt 2. It contains the original 25 package files and license.
- License: Apache-2.0, as included in the archive.

`pipeline.py known-good` first checks the committed fixture digest, then
compares the live registry download against it byte-for-byte AND against the
registry checksum. Two newly downloaded archives are never the comparison.
The fixture predates artifact attestations and is not represented as signed.
