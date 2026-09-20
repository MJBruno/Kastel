# Entiers : sémantique et débordement

## Règle

Les entiers Kastel sont des **entiers 64 bits signés** (de `-9223372036854775808` à `9223372036854775807`).
Un résultat qui sort de cet intervalle est une **erreur** (`IntegerOverflow`, capturable par `try/catch`), jamais un
nombre faux : avant, `+`, `-`, `*` et la négation « bouclaient » en silence (`factorial(21)` renvoyait
`-4249290049419214848`).

| Opération | Comportement |
|---|---|
| `a + b`, `a - b`, `a * b`, `-a` (entiers) | erreur `IntegerOverflow` si le résultat ne tient pas sur 64 bits |
| Entier et flottant mélangés | promotion en flottant (`9223372036854775807 + 1.0` → flottant), pas d'erreur |
| `a / b` | **toujours** un flottant ; erreur si `b == 0` |
| `a % b` | entier ; erreur si `b == 0` ; `i64::MIN % -1` vaut `0` |
| `idiv(a, b)` | division entière **exacte**, arrondie vers −∞ (`idiv(-7, 2)` = `-4`) ; erreur si `b == 0` ou `i64::MIN / -1` |
| `abs`, `pow` (deux entiers, exposant ≥ 0) | erreur `IntegerOverflow` si le résultat ne tient pas |
| `int(x)`, `floor(x)`, `ceil(x)`, `round(x)` | erreur `IntegerOverflow` si `x` (±∞ compris) sort de l'intervalle ; NaN → erreur de type. **Plus d'écrêtage silencieux** à `i64::MAX` |
| `wrapping_add`, `wrapping_sub`, `wrapping_mul` | arithmétique **volontairement cyclique** modulo 2^64 (hachage, générateurs pseudo-aléatoires) |

## Littéraux

* `9223372036854775807` est le plus grand littéral entier.
* `-9223372036854775808` (`i64::MIN`) est un littéral valide, alors que sa valeur absolue ne tient pas dans un `i64`.
* Un littéral plus grand est refusé avec un message explicite (« Entier trop grand… écrivez-le en flottant »).

## `std.math`

* `factorial(20)` = `2432902008176640000` est le plus grand factoriel exact ; `factorial(21)` lève `IntegerOverflow`.
* `fibonacci(92)` est exact ; `fibonacci(93)` lève `IntegerOverflow`. L'algorithme ne calcule plus `F(n+1)`, qui faisait
  échouer `fibonacci(92)` alors que le résultat tient.
* `lcm` divise **avant** de multiplier (`idiv(abs(a), pgcd) * abs(b)`) : il ne déborde que si le résultat lui-même déborde.
* `combinations` utilise `idiv` (exact) au lieu de `floor(x / y)`, qui perdait de la précision au-delà de 2^53.

## Pourquoi une erreur (et pas une promotion ou des grands entiers)

Une erreur est la seule option qui ne change ni le type ni la précision d'un résultat sans prévenir. La promotion
automatique en flottant fausserait silencieusement les valeurs au-delà de 2^53 ; les entiers de taille arbitraire
demandent une représentation dédiée et coûtent en performance. Ils restent possibles plus tard sans casser ce contrat
(un programme qui reçoit aujourd'hui `IntegerOverflow` obtiendrait alors un résultat exact).

## Limites connues

* La comparaison entier / flottant passe par `f64` : au-delà de 2^53 deux entiers distincts peuvent devenir « égaux »
  une fois comparés à un flottant.
* Les constantes de compilation (littéraux) ne sont pas repliées : un dépassement entre deux littéraux est levé à
  l'exécution, pas à la compilation.
