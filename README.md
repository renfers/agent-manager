# agent-manager

Moteur de workflow universel pour la constellation Anaréa.

## Vision

Un seul binaire Rust orchestre tous les flux de la constellation : messages Telegram, traduction du Livre de Yeshua, maturation des graines, notifications dashboard.

Chaque flux est un **workflow** déclaré en JSON. Le moteur est agnostique au métier. Il sait seulement qu'il manipule des objets avec des états, des transitions, et des hooks.

## Architecture

```
agent-manager/
├── src/
│   ├── main.rs              # Point d'entrée
│   ├── lib.rs               # Exports publics
│   ├── engine.rs            # Moteur de workflow (FSM)
│   ├── registry.rs          # Registre d'actions (native + wrapper)
│   ├── actions/             # Implémentations natives
│   │   ├── telegram.rs
│   │   ├── rate_limiter.rs
│   │   └── loopback.rs
│   ├── objects/             # Types d'objets workflow
│   │   └── message_telegram.rs
│   ├── store.rs             # Persistance SQLite
│   └── config.rs            # Lecture des JSON
├── tests/                   # Tests d'intégration
├── registry/                # Workflows déclaratifs
│   └── constellation-chat/
│       ├── states.json
│       ├── transitions.json
│       ├── hooks.json
│       └── config.json
└── wrappers/                # Scripts externes
    ├── call_hermes.py
    └── render_audio.sh
```

## Quick start

```bash
# Compiler
cargo build --release

# Lancer avec un workflow
./target/release/agent-manager --workflow constellation-chat

# Mode dry-run (ne publie jamais sur Telegram)
./target/release/agent-manager --workflow constellation-chat --dry-run
```

## Workflows disponibles

| Workflow | Description | Statut |
|----------|-------------|--------|
| `constellation-chat` | Routage messages Telegram + anti-emballement | En cours |
| `book-pipeline` | Traduction → relecture → publication Livre de Yeshua | À venir |
| `project-management` | Graine → Proposition → Projet | À venir |

## Documentation

- [`design/trait-action-registry.md`](design/trait-action-registry.md) — Interface Rust entre moteur et flux
- [`design/workflow-json-specs-v2.md`](design/workflow-json-specs-v2.md) — Spécification des 4 JSON

## Licence

MIT — Constellation Anaréa
