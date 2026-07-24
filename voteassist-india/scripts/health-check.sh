#!/usr/bin/env bash
# Curls every service's /healthz. Intended for a quick manual check or as
# a starting point for an external uptime monitor (Uptime Kuma, per
# docs/SECURITY-AND-SRE-OPERATIONS.md) — not a substitute for real
# Prometheus/Grafana alerting on the /metrics endpoints each service also
# exposes now.
#
# Override any address via environment variable (all default to
# localhost, matching each service's own default bind address).
set -uo pipefail

declare -A SERVICES=(
  [api]="${API_ADDR:-localhost:8080}"
  [web-app]="${WEB_APP_ADDR:-localhost:3000}"
  [admin-app]="${ADMIN_APP_ADDR:-localhost:3010}"
  [bot-whatsapp]="${BOT_WHATSAPP_ADDR:-localhost:8081}"
  [ivr-gateway]="${IVR_GATEWAY_ADDR:-localhost:8082}"
  [jobs]="${JOBS_HEALTH_ADDR:-localhost:9090}"
  [bot-telegram]="${BOT_TELEGRAM_HEALTH_ADDR:-localhost:9091}"
)

exit_code=0

for name in "${!SERVICES[@]}"; do
  addr="${SERVICES[$name]}"
  if response=$(curl -fsS --max-time 3 "http://$addr/healthz" 2>&1); then
    echo "OK    $name ($addr): $response"
  else
    echo "DOWN  $name ($addr): $response"
    exit_code=1
  fi
done

exit $exit_code
