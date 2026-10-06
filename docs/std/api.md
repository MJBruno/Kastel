# std 1.0 — API de référence

| Module | Rôle principal |
|---|---|
| `std.core` | types, invariants, combinators |
| `std.collections` | helpers List/Dict/Option/Result |
| `std.string` | utilitaires `str` |
| `std.math` | math sûres et génériques |
| `std.datetime` | date/heure UTC jusqu'à la milliseconde |
| `std.io` | console |
| `std.fs` | fichiers |
| `std.path` | chemins |
| `std.os` | environnement/processus |
| `std.process` | processus externes sans shell |
| `std.json` | JSON |
| `std.csv` | CSV |
| `std.toml` | TOML supporté |
| `std.yaml` | YAML supporté |
| `std.regex` | expressions régulières Unicode |
| `std.net` | TCP/UDP |
| `std.http` | client HTTP/1.1 |
| `std.thread` | tâches coopératives |
| `std.sync.*` | primitives de synchronisation |
| `std.statistics` | statistiques descriptives |
| `std.testing` | assertions et runner |
| `std.ops` | capabilities opérateurs |

Toutes les fonctions publiques sont documentées par leur signature. Les modules
qui traitent des données non fiables exposent des `Result` lorsque l'opération
peut échouer de façon normale.
