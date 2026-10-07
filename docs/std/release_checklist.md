# std 1.0.0 — release checklist

Avant de publier une version officielle :

1. `cargo check` sans avertissement.
2. `cargo test` sans échec.
3. `cargo tree` sans crate externe non approuvée.
4. `Cargo.toml` sans dépendance de la bibliothèque standard hors décision explicite du projet.
5. Tous les modules publics documentés.
6. Les signatures publiques correspondent aux contrats runtime.
7. Les anciens noms sont absents de la surface officielle ou documentés comme cassants dans `breaking.md`.
8. Le smoke test `tests_std_1_0.ks` s'exécute.
9. Windows et Linux sont testés séparément.
10. Le résultat des benchmarks de référence est archivé avant toute optimisation VM.
