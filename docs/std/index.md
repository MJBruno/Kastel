# Kastel Standard Library 1.0

Version: `1.0.0` — API stable.

## Modules officiels

`core`, `collections`, `string`, `math`, `datetime`, `io`, `fs`, `path`, `os`, `process`, `json`, `csv`, `toml`, `yaml`, `regex`, `net`, `http`, `thread`, `sync.*`, `statistics`, `testing`, `ops`.

## Règles d'API

- Les fonctions publiques sont typées quand le langage peut exprimer le contrat.
- Les erreurs récupérables utilisent `Result<T, str>`.
- L'absence d'une valeur utilise `Option<T>`.
- Les APIs système/réseau ne cachent pas les erreurs importantes.
- `List`, `size()`, `add()` et `contains()` sont les conventions officielles.
- `std.strings`, `std.file` et `std.statistic` ne font plus partie de l'API 1.0.

## Modules système

`std.fs` est dédié aux fichiers et `std.path` aux chemins. `std.os` est dédié à l'environnement du processus. `std.io` est dédié à la console.

## Données et texte

`std.json`, `std.csv`, `std.toml`, `std.yaml`, `std.regex` sont des APIs de parsing/sérialisation explicites sur `Result` lorsque l'entrée peut être invalide.

## Référence détaillée

- `core.md` — utilitaires fondamentaux
- `collections.md` — helpers de collections
- `string.md` — utilitaires texte
- `math.md` — mathématiques
- `datetime.md` — date/heure UTC et millisecondes
- `io.md` — console
- `fs.md` / `path.md` — système de fichiers et chemins
- `os.md` / `process.md` — environnement et processus
- `json.md` / `csv.md` / `toml.md` / `yaml.md` — formats de données
- `regex.md` / `regex_engine.md` — expressions régulières natives
- `net.md` / `http.md` — réseau et HTTP/1.1
- `thread.md` / `sync.md` — concurrence
- `statistics.md` — statistiques
- `testing.md` — tests
- `ops.md` — capabilities opérateurs
- `api.md` — matrice globale
- `migration.md` / `breaking.md` — migrations et changements cassants
- `versioning.md` — politique de versionnement
- `release_checklist.md` — contrôle avant publication
