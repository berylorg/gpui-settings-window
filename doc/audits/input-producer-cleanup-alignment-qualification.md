# Input Producer Cleanup Alignment Qualification

Settings consumes input revision `85223650b92f2b4dc65dd158a4ad0d94228dc7f8`, including
authenticated object-failure producer retirement. Its parent application must use this same
revision under the process-wide input action-registration contract. Only the manifest and
lockfile input revision change; settings source and features remain unchanged.

## Evidence

Parent Beryl retains the raw receipts under `.tmp/thread-lineage-evidence`. The two-path
snapshot `6B6B31B3667AF0D774C4EF80C43B6522ACF334FF38D56B07823785AE3EE9BF8C`
covers `Cargo.toml` and `Cargo.lock`.

- Canonical Git dependency locked metadata and all-target checking passed; checking finished
  in 7.03 seconds. The analyzer restart succeeded after validation.
- All 109 integration cases across eight binaries passed in 3.395 seconds, run
  `9ea7ec22-51e7-4357-a8d8-32432c1b11f3`.

- Independent dependency review verified exactly two revision substitutions, with no unrelated
  dependency, feature, source or adapter changes.

The alignment is accepted for publication. Beryl owns final unified resolution and native
recovery qualification.
