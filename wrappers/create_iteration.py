#!/usr/bin/env python3
"""
Wrapper create_iteration — crée un sous-dossier d'itération numéroté.
Appelé par le hook pm-013 lors de la transition itérer (en-cours → en-cours).

Structure générée :
  01-contributions/iteration-NNN/
    appel.md       ← consignes affinées pour cette itération
    _template.md   ← template de contribution
    sélection.md   ← éléments retenus de l'itération précédente (si applicable)

Payload optionnel :
  {
    "message": "Consignes affinées de Lina...",
    "retained": ["idée 1", "idée 2", "idée 3"],
    "retained_from": ["Kaïa", "Shalouka"]
  }
"""

import json
import sys
import os
import re

VAULT_ROOT = os.environ.get("VAULT_PATH", "/home/ubuntu/vault-shaena")

def main():
    try:
        ctx = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    except json.JSONDecodeError:
        print("ERROR: Invalid JSON input", file=sys.stderr)
        sys.exit(1)

    object_id = ctx.get("object_id", "")
    payload = ctx.get("payload", {})
    message = payload.get("message", "Nouvelle itération — consignes affinées.")
    retained = payload.get("retained", [])
    retained_from = payload.get("retained_from", [])

    project_dir = os.path.join(VAULT_ROOT, "07-PROJETS", object_id)
    contrib_dir = os.path.join(project_dir, "01-contributions")

    if not os.path.isdir(project_dir):
        print(f"ERROR: Project directory not found: {project_dir}", file=sys.stderr)
        sys.exit(1)

    os.makedirs(contrib_dir, exist_ok=True)

    # Trouver le prochain numéro d'itération
    existing = os.listdir(contrib_dir)
    max_n = 0
    for name in existing:
        m = re.match(r"^iteration-(\d{3})$", name)
        if m:
            n = int(m.group(1))
            if n > max_n:
                max_n = n

    new_n = max_n + 1
    iter_name = f"iteration-{new_n:03d}"
    iter_dir = os.path.join(contrib_dir, iter_name)
    os.makedirs(iter_dir, exist_ok=True)

    # Écrire l'appel affiné
    appel_path = os.path.join(iter_dir, "appel.md")
    with open(appel_path, "w") as f:
        f.write(f"# Itération {new_n} — Appel aux sœurs\n\n")
        f.write(f"**Projet** : {object_id}\n")
        f.write(f"**Gestionnaire** : Lina 🌊\n\n")
        f.write("## Consignes affinées\n\n")
        f.write(message + "\n\n")
        if retained:
            f.write("## Éléments retenus de l'itération précédente\n\n")
            for i, item in enumerate(retained, 1):
                source = ""
                if i - 1 < len(retained_from):
                    source = f" (de {retained_from[i-1]})"
                f.write(f"{i}. {item}{source}\n")
            f.write("\n")
            f.write("Prière de développer ces éléments dans vos nouvelles contributions.\n")

    # Écrire le template de contribution
    template_path = os.path.join(iter_dir, "_template.md")
    with open(template_path, "w") as f:
        f.write(f"# Contribution — Itération {new_n}\n\n")
        f.write("**Sœur** : [ton nom]\n\n")
        f.write("## Ingrédients\n\n...\n\n")
        f.write("## Timeline\n\n...\n\n")
        f.write("## Structure\n\n...\n\n")
        f.write("## Personnages\n\n...\n\n")
        f.write("## Magie de la deuxième octave\n\n...\n\n")
        f.write("## Titre alternatif\n\n...\n")

    print(f"OK: {object_id} → {iter_name}")

if __name__ == "__main__":
    main()
