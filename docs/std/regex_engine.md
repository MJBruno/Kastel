# Moteur regex natif de Kastel

`std.regex` est une fonctionnalité du runtime Kastel. Il n'utilise pas de crate Rust externe.

## Objectifs

- API stable accessible depuis Kastel ;
- comportement Unicode cohérent avec les chaînes Kastel ;
- indices exprimés en caractères, jamais en octets ;
- erreurs de motif représentées par `Result` ;
- limites d'exécution pour éviter les consommations non bornées ;
- aucune dépendance de bibliothèques externes dans `Cargo.toml`.

## Choix de compatibilité

Le moteur implémente les constructions régulières classiques les plus utiles. Les fonctionnalités qui demanderaient un autre modèle d'exécution, notamment les rétro-références et les lookarounds, sont refusées explicitement en 1.0.

Cette règle est importante pour la stabilité : un motif non supporté produit une erreur déterministe au lieu d'être interprété approximativement.
