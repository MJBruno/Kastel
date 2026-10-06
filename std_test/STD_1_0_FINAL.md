# Kastel Standard Library 1.0.0

## Statut

`std` 1.0.0 est la première API officielle stable. Les anciens modules et noms
expérimentaux peuvent être supprimés sans compatibilité rétroactive.

## Modules

`core`, `collections`, `string`, `math`, `datetime`, `io`, `fs`, `path`, `os`,
`process`, `json`, `csv`, `toml`, `yaml`, `regex`, `net`, `http`, `thread`,
`sync.*`, `statistics`, `testing`, `ops`.

## Intégration runtime

Les fichiers Rust supplémentaires de cette distribution sont :

- `src/stdlib/regex.rs`
- `src/stdlib/mod.rs`
- `src/compiler/builtin_types.rs`

Le moteur regex utilise la dépendance Rust `regex = "1"`. Ajouter cette ligne
au `[dependencies]` du manifeste du projet si le `Cargo.toml` local contient
d'autres réglages spécifiques au dépôt.

## Validation

Depuis le dépôt Kastel :

```powershell
cargo check
cargo test
cargo run -q --bin kastel tests_std_1_0.ks
```

## Principes

- signatures publiques explicites ;
- `Result` pour les erreurs récupérables ;
- `Option` pour l'absence ;
- séparation nette fichier/chemin/JSON ;
- API réseau explicite ;
- HTTP 1.1 client séparé de TLS ;
- documentation et tests de régression ;
- API cassante assumée avant gel 1.0.
