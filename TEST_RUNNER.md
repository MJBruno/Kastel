# Kastel Test Runner

Ce runner remplace les scripts PowerShell/Bash de la suite de tests.

## Structure

```text
test_runner/
├── Cargo.toml
└── src/
    └── main.rs

test/
├── regression/
└── errors/
```

## Lancement depuis la racine de Kastel

```powershell
cargo run --manifest-path test_runner/Cargo.toml --
```

Le runner :

1. cherche automatiquement la racine du projet Kastel ;
2. cherche `target/debug/kastel` ;
3. lance `cargo build --bin kastel` si l'exécutable n'existe pas ;
4. découvre tous les `.ks` sous `test/regression` et `test/errors` ;
5. retire le chronométrage `Process finished...` des sorties ;
6. normalise CRLF/LF et les séquences ANSI ;
7. compare les régressions avec leurs fichiers `.expected` ;
8. vérifie que les tests d'erreur terminent avec un code non nul et un message exploitable ;
9. affiche le résultat et le temps de chaque test ;
10. retourne `0` si tout passe, `1` si au moins un test échoue, `2` pour une erreur du runner.

## Filtres

Régressions uniquement :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --regression
```

Erreurs uniquement :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --errors
```

Un test ou groupe de tests :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --filter iterator
```

Mode release :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --release
```

Sortie détaillée :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --verbose
```

Binaire explicite :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --binary .\target\debug\kastel.exe
```

## Mise à jour contrôlée des expected

Le runner n'écrase jamais les attentes par défaut.

Après avoir inspecté manuellement le comportement :

```powershell
cargo run --manifest-path test_runner/Cargo.toml -- --filter nom_du_test --bless
```

`--bless` est donc volontairement explicite.

## Vérification

Compiler uniquement le runner :

```powershell
cargo check --manifest-path test_runner/Cargo.toml
```

Puis l'exécuter :

```powershell
cargo run --manifest-path test_runner/Cargo.toml --
```
