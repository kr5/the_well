#!/usr/bin/env bash
# Post-deploy smoke test: goes one step past health-check.sh's plain
# liveness check by actually exercising a handful of real, cheap
# endpoints and checking their response looks right — not just that the
# process answered. Run this right after a deploy/restart, before
# declaring it done.
#
# Override any address via the same environment variables
# health-check.sh uses (all default to localhost, matching each
# service's own default bind address).
set -uo pipefail

API_ADDR="${API_ADDR:-localhost:8080}"
WEB_APP_ADDR="${WEB_APP_ADDR:-localhost:3000}"
ADMIN_APP_ADDR="${ADMIN_APP_ADDR:-localhost:3010}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

exit_code=0

echo "== Liveness (scripts/health-check.sh) =="
if ! "$SCRIPT_DIR/health-check.sh"; then
  exit_code=1
fi

check_status() {
  local description="$1" method="$2" url="$3"
  local status
  status=$(curl -fsS -o /dev/null -w '%{http_code}' --max-time 5 -X "$method" "$url" 2>/dev/null) || status="curl failed"
  if [[ "$status" =~ ^2 ]]; then
    echo "OK    $description ($url): HTTP $status"
  else
    echo "FAIL  $description ($url): $status"
    exit_code=1
  fi
}

check_body_contains() {
  local description="$1" url="$2" expected="$3"
  local body
  if body=$(curl -fsS --max-time 5 "$url" 2>&1) && [[ "$body" == *"$expected"* ]]; then
    echo "OK    $description ($url): response contains \"$expected\""
  else
    echo "FAIL  $description ($url): response did not contain \"$expected\""
    exit_code=1
  fi
}

echo
echo "== Functional checks =="
check_status "api: start a decision-engine session" POST "http://$API_ADDR/v1/sessions"
check_status "api: search the knowledge base" GET "http://$API_ADDR/v1/kb/search?q=epic"
check_status "api: Swagger UI" GET "http://$API_ADDR/docs"
# Every rendered page carries the "we are not the ECI" banner
# (NOT_OFFICIAL_BANNER_TEXT, docs/06-legal-compliance-review.md Section 1)
# — checking for a distinctive phrase from it catches a "200 OK but the
# page is actually broken/blank" false positive a bare status check would
# miss, without being fragile to unrelated copy edits elsewhere on the page.
check_body_contains "web-app: homepage renders" "http://$WEB_APP_ADDR/" "Election Commission of India"
check_status "admin-app: login page" GET "http://$ADMIN_APP_ADDR/login"

echo
if [ "$exit_code" -eq 0 ]; then
  echo "All smoke tests passed."
else
  echo "One or more smoke tests failed — see FAIL lines above." >&2
fi

exit $exit_code
