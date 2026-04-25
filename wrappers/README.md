# Wrappers

Scripts externes appelés par le moteur via `ScriptWrapper`.

## Contrat

Chaque script reçoit sur **stdin** un JSON :
```json
{
  "hook": { "action": "...", "payload": { ... } },
  "context": { "workflow_id": "...", "object_id": "...", "triggered_by": "..." },
  "object": { ... }
}
```

Et doit écrire sur **stdout** un JSON `HookSignal` :
```json
{
  "signal": "Continue",
  "next_action": null,
  "context": { ... }
}
```

## Liste des wrappers

| Nom | Script | Langage | Description |
|-----|--------|---------|-------------|
| `call_hermes` | `call_hermes.py` | Python 3 | Appelle Hermes sur Ubuntu |
| `read_vault` | `read_vault.py` | Python 3 | Lit un fichier du vault Obsidian |

## Timeout

Le moteur tue le script après le `timeout_seconds` défini dans `config.json`.

## Stderr

Redirigé dans les logs du moteur (niveau `Debug`).
