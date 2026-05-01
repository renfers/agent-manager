#!/usr/bin/env python3
"""
Wrapper notify_watchdog — notifie le watchdog constellation d'un changement d'état.
Appelé par le moteur agent-manager quand les hooks pm-003/006/009/011 se déclenchent.

Input (argv[1]) : JSON avec le contexte d'action
Output : POST vers le SSE endpoint du watchdog (port 18800)
"""

import json
import sys
import os
import urllib.request

WATCHDOG_SSE = os.environ.get("WATCHDOG_SSE_URL", "http://127.0.0.1:18800")

def main():
    try:
        ctx = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    except json.JSONDecodeError:
        print("ERROR: Invalid JSON input", file=sys.stderr)
        sys.exit(1)

    object_id = ctx.get("object_id", "")
    target_state = ctx.get("target_state", "")
    payload = ctx.get("payload", {})
    event = payload.get("event", "project-updated")

    # Envoyer une notification au watchdog via SSE (log uniquement pour l'instant)
    data = json.dumps({
        "object_id": object_id,
        "state": target_state,
        "event": event,
    })

    try:
        # Le watchdog écoute sur le port SSE — on logue l'événement
        # car le watchdog utilise un mécanisme pull (PollingObserver) et non push
        watchdog_state = os.path.join(
            os.environ.get("VAULT_PATH", "/home/ubuntu/vault-shaena"),
            ".watchdog_events.log"
        )
        with open(watchdog_state, "a") as f:
            f.write(f"[{event}] {object_id} → {target_state}\n")
        print(f"OK: {event} {object_id}")
    except Exception as e:
        print(f"WARN: Could not notify watchdog: {e}", file=sys.stderr)
        # Non-fatal — le watchdog détecte les changements via PollingObserver

if __name__ == "__main__":
    main()
