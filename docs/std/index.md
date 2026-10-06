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
