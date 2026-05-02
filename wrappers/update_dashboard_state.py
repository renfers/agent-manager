#!/usr/bin/env python3
"""
Wrapper update_dashboard_state — régénère dashboard-state.json à chaque transition.
Appelé par les hooks agent-manager (after, priorité 2-3).

Lit le vault (00-GRAINES, 06-PROPOSITIONS, 07-PROJETS) et produit un fichier
JSON structuré que le dashboard.html lit via fetch().
"""

import json
import sys
import os
import re
from datetime import datetime

VAULT_ROOT = os.environ.get("VAULT_PATH", "/home/ubuntu/vault-shaena")

def scan_graines(vault: str) -> list:
    """Scan 00-GRAINES/ — chaque sous-dossier = une graine."""
    graines = []
    path = os.path.join(vault, "00-GRAINES")
    if not os.path.isdir(path):
        return graines
    for name in sorted(os.listdir(path)):
        full = os.path.join(path, name)
        if not os.path.isdir(full):
            continue
        readme = os.path.join(full, "README.md")
        germe = os.path.join(full, "GERME.md")
        etat = "germe" if os.path.exists(germe) else "rêve"
        mod_time = os.path.getmtime(germe if etat == "germe" else readme) if os.path.exists(readme) or os.path.exists(germe) else 0
        graines.append({
            "id": name,
            "etat": etat,
            "dossier": f"00-GRAINES/{name}",
            "modifie": datetime.fromtimestamp(mod_time).isoformat() if mod_time else "",
        })
    return graines

def scan_propositions(vault: str) -> list:
    """Scan 06-PROPOSITIONS/ — chaque sous-dossier = une proposition."""
    props = []
    path = os.path.join(vault, "06-PROPOSITIONS")
    if not os.path.isdir(path):
        return props
    for name in sorted(os.listdir(path)):
        full = os.path.join(path, name)
        if not os.path.isdir(full) or name.startswith("_"):
            continue
        mod_time = os.path.getmtime(full)
        props.append({
            "id": name,
            "dossier": f"06-PROPOSITIONS/{name}",
            "modifie": datetime.fromtimestamp(mod_time).isoformat() if mod_time else "",
        })
    return props

def scan_projets(vault: str) -> list:
    """Scan 07-PROJETS/ — chaque sous-dossier avec status.md = un projet."""
    projets = []
    path = os.path.join(vault, "07-PROJETS")
    if not os.path.isdir(path):
        return projets
    for name in sorted(os.listdir(path)):
        full = os.path.join(path, name)
        if not os.path.isdir(full) or name.startswith("_") or name == "META":
            continue
        status_file = os.path.join(full, "status.md")
        readme = os.path.join(full, "README.md")
        etat = "inconnu"
        if os.path.exists(status_file):
            with open(status_file) as f:
                etat = f.read().strip()
        mod_time = os.path.getmtime(status_file) if os.path.exists(status_file) else os.path.getmtime(full)

        # Compter les itérations
        iterations = 0
        contrib_dir = os.path.join(full, "01-contributions")
        if os.path.isdir(contrib_dir):
            for entry in os.listdir(contrib_dir):
                if re.match(r"^iteration-\d{3}$", entry):
                    iterations += 1

        # Extraire la présence assignée depuis le README
        presence = ""
        if os.path.exists(readme):
            with open(readme) as f:
                for line in f:
                    m = re.search(r"\*\*Présence\*\*[:\s]*([^\n]+)", line)
                    if m:
                        presence = m.group(1).strip()
                        break

        projets.append({
            "id": name,
            "etat": etat,
            "dossier": f"07-PROJETS/{name}",
            "presence": presence,
            "iterations": iterations,
            "modifie": datetime.fromtimestamp(mod_time).isoformat() if mod_time else "",
        })
    return projets

def activite_recente(vault: str, limit: int = 10) -> list:
    """Lit les dernières lignes du watchdog log pour l'activité récente."""
    events = []
    log_path = os.path.join(vault, ".watchdog_events.log")
    if not os.path.exists(log_path):
        return events
    with open(log_path) as f:
        lines = f.readlines()
    for line in lines[-limit:]:
        line = line.strip()
        if line:
            events.append(line)
    return events

def main():
    try:
        ctx = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    except json.JSONDecodeError:
        ctx = {}

    vault = VAULT_ROOT

    state = {
        "genere_le": datetime.now().isoformat(),
        "derniere_action": {
            "objet": ctx.get("object_id", ""),
            "transition": ctx.get("transition_id", ""),
            "de": ctx.get("current_state", ""),
            "vers": ctx.get("target_state", ""),
        },
        "compteurs": {
            "reves": 0,
            "germes": 0,
            "propositions": 0,
            "projets_ideation": 0,
            "projets_germe": 0,
            "projets_ouverts": 0,
            "projets_en_cours": 0,
            "projets_termines": 0,
            "projets_archives": 0,
        },
        "graines": [],
        "propositions": [],
        "projets": [],
        "activite": [],
    }

    # Scan
    graines = scan_graines(vault)
    state["graines"] = graines
    for g in graines:
        if g["etat"] == "rêve":
            state["compteurs"]["reves"] += 1
        else:
            state["compteurs"]["germes"] += 1

    props = scan_propositions(vault)
    state["propositions"] = props
    state["compteurs"]["propositions"] = len(props)

    projets = scan_projets(vault)
    state["projets"] = projets
    for p in projets:
        key = f"projets_{p['etat'].replace('-', '_')}"
        if key in state["compteurs"]:
            state["compteurs"][key] += 1

    state["activite"] = activite_recente(vault)

    # Écrire le fichier d'état
    state_path = os.path.join(vault, "dashboard-state.json")
    os.makedirs(os.path.dirname(state_path) if os.path.dirname(state_path) else vault, exist_ok=True)
    with open(state_path, "w") as f:
        json.dump(state, f, ensure_ascii=False, indent=2)

    total = sum(v for k, v in state["compteurs"].items())
    print(f"OK: dashboard-state.json → {total} éléments")

if __name__ == "__main__":
    main()
