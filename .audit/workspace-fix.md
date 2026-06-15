# PlayCua Workspace Fragmentation Fix — 2026-06-14

**Repo:** `/Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua/`

## Issue (from prior audit)

Two crates — `cli-wrapper` and `playcua-bare` — are not members of the root `Cargo.toml` workspace. They have their own `[workspace]` declarations and are not picked up by `cargo build --workspace`. Additionally, `cli-wrapper/Cargo.toml` has a brittle external path dep on `pheno-cli-base` via `../../../pheno-cli-base`.

## Fix Plan

### Step 1: Add the missing crates to root workspace

Edit `/Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua/Cargo.toml` and add to the `members` array (or list, depending on syntax):

```toml
[workspace]
members = [
    "crates/playcua-core",
    "crates/cli-wrapper",
    "crates/playcua-bare",
    # ... other existing members
]
```

If the workspace uses a glob pattern like `members = ["crates/*"]`, this is already covered. Verify by running:

```bash
grep -A 10 "^\[workspace\]" /Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua/Cargo.toml
```

### Step 2: Remove nested workspace declarations

In each of `crates/cli-wrapper/Cargo.toml` and `crates/playcua-bare/Cargo.toml`, remove the `[workspace]` block (if it has one). This is needed because nested workspaces are not allowed in a parent workspace context.

### Step 3: Replace brittle path dep

In `crates/cli-wrapper/Cargo.toml`, change:

```toml
[dependencies]
pheno-cli-base = { path = "../../../pheno-cli-base" }
```

to either:

- (a) **Git dep** (if `pheno-cli-base` is published): `pheno-cli-base = "0.1"`
- (b) **Workspace dep** (if it's in the same monorepo): `pheno-cli-base = { workspace = true }`
- (c) **Path with relative fix**: `pheno-cli-base = { path = "../../pheno-cli-base" }` (correct the depth)

### Step 4: Verify

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/PlayCua
cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]' | grep -E "(cli-wrapper|playcua-bare)"
cargo build --workspace
cargo test --workspace
```

## Estimated Time

- Workspace member fix: ~10 min
- Path dep replacement: ~15 min
- Build + test verification: ~30 min (PlayCua is large)
- **Total: ~55 min**
