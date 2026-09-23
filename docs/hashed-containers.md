# `Dict` et `Set` : recherche en O(1)

`Dict` et `Set` reposaient sur un `Vec` parcouru linéairement (O(n) par
recherche, insertion ou appartenance ; O(n²) pour construire une collection
de n éléments). Ils passent désormais par `runtime/hashed.rs`
(`DictEntries`, `SetElements`) : un index de hachage accompagne le `Vec`,
qui reste la source de vérité de l'**ordre d'insertion** (toujours conservé,
mais toujours hors contrat).

## Égalité et hachage des clés

Communs à `Dict` (clés) et `Set` (éléments) — `Value::key_equals` /
`Value::key_hash` :

* nombres : `1` et `1.0` sont la **même** clé, comparés **exactement**
  (pas via `f64` au-delà de 2^53 — `Value::equals` sur `Integer`/`Float` a
  été corrigé au passage : il n'était plus exact pour les grands entiers) ;
* chaînes et tuples : par **contenu** (les tuples sont immuables, donc sûrs
  à comparer ainsi) ;
* tout le reste (liste, dict, record, ensemble, fonction, instance) : par
  **identité**.

## Complexité

| Opération | Avant | Maintenant |
|---|---|---|
| `contains`, `get`, `add`/`set` | O(n) | O(1) moyen |
| `remove` | O(n) | O(n) (décalage des positions, comme un `Vec::remove`) |
| Construire n éléments | O(n²) | O(n) |

## Points d'attention

* Une valeur peut être sa propre clé (`d[d] = 1`, `s.add(s)`) sans paniquer :
  le hachage et la comparaison se font **avant** l'emprunt en écriture du
  conteneur.
* Un objet mutable (liste, dict...) utilisé comme clé est hachée par
  **identité** : le modifier après coup n'invalide pas sa position dans la
  table (contrairement à une clé hachée par contenu, qui deviendrait
  introuvable si son contenu changeait).
