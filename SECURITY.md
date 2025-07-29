# Security Policy

The following is the security policy for:

- The `omnibor` library crate.
- The `omnibor_cli` binary crate.

All of which are found in this workspace.

## Reporting a Vulnerability

Vulnerabilities can be reported using the "Report a Vulnerability" button under
the security tab of the repository. If a vulnerability is found to be legitimate,
a RustSec advisory will be created.

Please give us 90 days to respond to a vulnerability disclosure. In general, we
will try to be faster than that to produce fixes and respond publicly to
disclosures.

If we accept the legitimacy of a vulnerability, please wait for us to have
publcily responded to the vulnerability, including publication of new versions,
yanking of old versions, and public disclosure in the RustSec database, before
publicly disclosing the vulnerability yourself.

We ask that you _not_ create advisories yourself, but instead submit
vulnerability reports to us first so we can plan a response including
producing any necessary patches, publishing fixed versions, yanking affected
versions, and communicating about the vulnerability to users.

We consider soundness violations (violations of safe Rust's memory, thread, or
type safety guarantees) to be at least informational vulnerabilities and
will treat them as such.

RustSec advisories are automatically imported into the GitHub Security Advisory
system, and into the OSV database, so duplicate reports do not need to be made
for those systems.
