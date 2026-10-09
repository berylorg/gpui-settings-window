# Input Revision Alignment Qualification

Settings consumes input revision `9adf033a8f3c46a9e1332bec47e816adf4085131`, matching its
parent application's qualified input dependency. Linking different input revisions registers
duplicate process-wide GPUI actions. The correction changes only the manifest and lockfile
revision; settings source, APIs, features and behavior remain unchanged.

## Evidence

The two-path dependency snapshot `92E6FC5396FCC9CD114415C105FE7B620AB8FD3494ED0EC3175087CB8B92F1D6`
covers `Cargo.toml` and `Cargo.lock`. Parent Beryl retains raw receipts under
`.tmp/thread-lineage-evidence`.

- Locked metadata and all-target checking passed against the canonical Git input dependency;
  checking finished in 7.69 seconds. The analyzer restart succeeded after validation.
- All 109 integration cases across eight binaries passed in 3.488 seconds, run
  `9f661092-c719-4bd8-ba66-67da0b1dc4db`.
- Independent dependency review verified both frozen hashes and exactly two revision
  substitutions, with no unrelated dependency, feature, settings source or adapter changes.

The dependency alignment is accepted for publication. Beryl owns its resulting settings pin,
single-input-package resolution and native recovery qualification.
