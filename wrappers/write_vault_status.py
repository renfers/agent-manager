#!/usr/bin/env python3
"""
Wrapper write_vault_status — écrit le statut d'un projet dans le vault 07-PROJETS/
Appelé par le moteur agent-manager quand le hook pm-002/004/005/008/010 se déclenche.

Input (argv[1]) : JSON avec {object_id, workflow_id, current_state, target_state, payload}
Output : met à jour status.md dans le dossier du projet
"""

import json
import sys
import os

VAULT_ROOT = os.environ.get(
    "VAULT_PATH",
    "/home/ubuntu/vault-shaena"
)

def main():
    try:
        ctx = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    except json.JSONDecodeError:
        print("ERROR: Invalid JSON input", file=sys.stderr)
        sys.exit(1)

    object_id = ctx.get("object_id", "")
    target_state = ctx.get("target_state", "")
    payload = ctx.get("payload", {})
    status = payload.get("status", target_state)

    project_dir = os.path.join(VAULT_ROOT, "07-PROJETS", object_id)
    os.makedirs(project_dir, exist_ok=True)

    status_file = os.path.join(project_dir, "status.md")
    with open(status_file, "w") as f:
        f.write(status + "\n")

    print(f"OK: {object_id} → {status}")

if __name__ == "__main__":
    main()
