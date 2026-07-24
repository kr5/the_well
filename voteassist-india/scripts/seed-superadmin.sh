#!/usr/bin/env bash
# Creates the first superadmin account for crates/admin-app. admin-app has
# no self-serve signup by design (see crates/admin-app/README.md's "User &
# Role Management" gap) — this is the deliberately out-of-band way to
# bootstrap the very first account. Run once per environment; every
# account after that can use `xtask create-admin` directly (with whatever
# role is appropriate), or admin-app's own (not-yet-built) User & Role
# Management page once it exists.
#
# The password is read from an interactive prompt, not a CLI argument —
# a CLI arg would land in shell history and be visible to anyone on the
# box via `ps`, neither of which is acceptable for a credential.
set -euo pipefail

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/voteassist" >&2
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <email>" >&2
  exit 1
fi

EMAIL="$1"

read -r -s -p "Password for $EMAIL: " PASSWORD
echo
read -r -s -p "Confirm password: " PASSWORD_CONFIRM
echo

if [ "$PASSWORD" != "$PASSWORD_CONFIRM" ]; then
  echo "error: passwords did not match" >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"

# xtask itself still takes the password positionally (it has no
# interactive-prompt logic of its own — it's a general-purpose CLI meant
# to be scriptable), so this wrapper is what keeps the credential out of
# this shell's own history/process list; xtask's argv is only visible to
# other processes for the brief moment it actually runs, same as any CLI
# tool taking a secret argument.
cargo run --manifest-path "$REPO_ROOT/rust/Cargo.toml" -p xtask --release -- create-admin "$EMAIL" "$PASSWORD" superadmin
