# Scope

Align the canonical widget graph with Beryl's accepted checked clipboard boundary.

# Phase 1: Align Canonical GPUI Dependency Graph (wip)

Prepared GPUI edd4928c5be424630da49f872e00dafbf94cf0b2, scrollbar
b9e591820b61fc788f148bca1f6f80b6341a692c and text-input
7f645bf2837072633d613fd47694898af5b218fc. Canonical locked metadata, all-target compilation and
independent semantic review pass. Only the thirteen affected Git package sources changed;
features, versions and dependency edges remain unchanged. Manifest/lockfile remain working material.

Blocked on 2026-10-05: after Operator manually restarted Serena, the first language-server restart
returned OK. The required restart after settings compilation timed out after 120 seconds.
Stop implementation; restore Serena and obtain a successful restart before relying on the changed
Cargo model or publishing alignment. Beryl's canonical consumer checks remain separate.
