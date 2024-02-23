# `xtask`

This is the `xtask` package for the OmniBOR Rust project. This implements
commonly-used project-wide commands for convenience.

## Design Goals

This crate has a few key design goals:

- __Fast compilation__: This tool will get recompiled whenever changes are
  made to it, and we want to empower contributors to the OmniBOR project to
  make changes to `xtask` when they encounter a new task for the project that
  they want to automate. To make this editing appealing, the write-edit-run
  loop needs to be fast, which means fast compilation.
- __Minimal dependencies__: Related to the above, the `xtask` crate should
  have a minimal number of dependencies, and where possible those dependencies
  should be configured with the minimum number of features.
- __Easy to use__: The commands exposed by `xtask` should have as simple an
  interface, and be as automatic, as possible. Fewer flags, fewer required
  arguments, etc.
