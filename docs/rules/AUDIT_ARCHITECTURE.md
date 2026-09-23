# Audit d'architecture — points faibles de Kastel

> Méthode : lecture statique du dépôt `Kastel-corrige` (≈ 27 700 lignes de Rust).
> **Rien n'a été compilé ni exécuté** (pas de toolchain Rust disponible). Les « reproducteurs » ci-dessous
> sont déduits du code, à confirmer sur votre machine. Références : `fichier:zone`.

## Statut des points critiques

Les points **1, 2 et 3** ont été traités (voir `docs/robustness.md` et `docs/integers.md`) ; les autres restent ouverts.
Toujours **non compilé** : à valider avec `cargo check` puis `cargo test`.

## Synthèse

| # | Gravité | Point faible | Nature |
|---|---|---|---|
| 1 | ✅ Corrigé (à valider) | Racines du GC incomplètes : des objets vivants pouvaient être **vidés** | Correction (perte de données silencieuse) |
| 2 | ✅ Corrigé (à valider) | Aucune limite de profondeur : récursion infinie = mémoire sans fin ; affichage cyclique = plantage | Robustesse |
| 3 | ✅ Corrigé (à valider) | Entiers 64 bits qui **débordaient en silence** | Sémantique |
| 4 | 🟠 Élevée | Plusieurs « sources de vérité » à tenir alignées à la main | Maintenabilité |
| 5 | 🟠 Élevée | Typage statique limité à un fichier pour les classes | Conception |
| 6 | 🟠 Élevée | Couverture de tests faible sur VM / parser / compilateur, pas d'intégration continue | Qualité |
| 7 | 🟡 Moyenne | Coûts de performance structurels (chaînes allouées, recherches linéaires) | Performance |
| 8 | 🟡 Moyenne | Désucrage dans le parser : l'AST ne reflète plus le source | Conception |
| 9 | 🟡 Moyenne | Visibilité `private` : plusieurs trous de sémantique | Conception |
| 10 | 🟡 Moyenne | Limites dures (255 éléments, 255 locales, 64 Ko de saut…) et erreurs peu précises | Ergonomie |
| 11 | 🟢 Faible | Fichiers trop gros, REPL limité, mémoire des positions | Dette |

---

## 1. 🔴 GC : racines incomplètes → objets vivants vidés

**Mécanisme.** Le GC est un *mark & sweep* posé sur des `Rc<RefCell<…>>`. Le « sweep » ne libère pas : pour tout objet
enregistré et **non marqué**, il appelle `break_cycle()`, qui **vide son contenu** (`Array::clear()`, `Dict::clear()`,
`Set::clear()`…) (`runtime/gc.rs`, phase sweep). La correction dépend donc de la **précision absolue des racines**.
Or le registre est global (`thread_local! REGISTRY`) alors que les racines viennent d'**une seule** VM
(`vm/machine/gc.rs` : pile, globales, frames, upvalues, modules chargés).

**Trous identifiés.**

1. **Temporaires Rust pendant un rappel.** `invoke_array_functional` (`vm/machine/methods.rs`, ~l. 643) accumule
   `result: Vec<Value>` dans une variable Rust, puis appelle `invoke_sync` pour chaque élément. Or `invoke_sync`
   (`vm/machine/calls.rs`, ~l. 171) teste `gc::should_collect()` à **chaque instruction**. Les résultats déjà produits
   ne sont sur aucune racine → ils peuvent être vidés. Même schéma pour `filter`, `reduce`, `any`, `all`, et les
   méthodes d'itérateur (`vm/machine/iterators.rs`).
2. **VM imbriquées.** `import` exécute le module dans une **nouvelle VM** (`VirtualMachine::execute_module`).
   Si le GC se déclenche dans cette VM, la pile et les globales du programme appelant ne sont **pas** des racines.
3. **Récepteur temporaire.** Pour `f().map(cb)` ou `[…].map(cb)`, le tableau récepteur peut n'être tenu que par des
   variables Rust (`args`, `elements`) pendant les rappels.

**Reproducteur probable** (seuil initial : 256 allocations) :
```kastel
let a = [];
for i in range(0, 2000) { a.add(i); }
let pairs = a.map(func(x) { return [x, x * 2]; });
println(pairs.get(0));      // attendu : [0, 0] — risque : []
```

**Correctif proposé (petit).**
* Ajouter à `VirtualMachine` un compteur `gc_inhibit` (ou une liste `temp_roots` incluse dans `GcRoots`) ; les
  fonctions natives qui rappellent du code Kastel l'incrémentent, et `invoke_sync` ne collecte pas tant qu'il est > 0.
* Dans `execute_module`, utiliser `run_without_gc()` (existe déjà, marqué `dead_code`) ou enregistrer les racines
  des VM actives dans une pile `thread_local`.
* Long terme : remplacer le « vidage » par une vraie libération, ou ne casser que les objets dont le comptage
  strict prouve un cycle.

---

## 2. 🔴 Aucune limite de profondeur / de cycles

* **Récursion Kastel** : `frames.push(...)` (`vm/machine/calls.rs` ~l. 55) n'a aucune limite. `func f(n) { return f(n+1); }`
  consomme la mémoire jusqu'à épuisement au lieu de lever une erreur `StackOverflow` capturable.
* **Récursion Rust** : `invoke_sync` relance une boucle d'exécution imbriquée ; l'affichage (`Value::fmt_sequence`),
  `json_encode`, `inspect`, le parser et le vérificateur de types sont récursifs **sans garde**.
  `let a = []; a.add(a); println(a);` ou une expression imbriquée sur des milliers de niveaux → débordement de pile
  natif (arrêt brutal, non rattrapable par `try/catch`).

**Correctif proposé.** Constante `MAX_CALL_DEPTH` (ex. 10 000) → `RuntimeError::StackOverflow` ; garde de profondeur
dans `fmt_repr`/JSON/inspect (`[...]` pour un cycle) ; profondeur maximale d'imbrication dans le parser.

---

## 3. 🟠 Entiers : débordement silencieux

`runtime/value.rs` (~l. 1060) : `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg`. Exemple concret dans
la bibliothèque standard : `math.factorial(21)` renvoie `-4249290049419214848` sans erreur ; idem `fibonacci(93)`.

**Options** : erreur `IntegerOverflow` (le plus sûr), promotion automatique en flottant, ou entiers arbitraires.
À décider *avant* de figer la sémantique du langage.

---

## 4. 🟠 Plusieurs sources de vérité à aligner à la main

| Ce qu'on ajoute | Endroits à modifier |
|---|---|
| Une fonction native | `stdlib/*.rs` (implémentation + `register` + `register_compiler`) **et** `compiler/builtin_types.rs` |
| Une méthode de collection | `stdlib/<type>.rs::dispatch_method` **et** `compiler/types.rs` (`collection_member_type`, `set_member_type`, `renamed_member`) |
| Une variante d'`OpCode` / `Object` / `RuntimeError` / `CompileError` / `Type` | 3 à 5 `match` exhaustifs (dispatch, désassembleur, `Display`, `Diagnostic`, classification dans `vm/machine/execution.rs`, GC, `type_name`…) |

Preuves de dérive déjà rencontrées : `rand_range` typé à 1 argument alors que la native en prend 2 ; `OpCode::Wide` émis
par le compilateur mais absent de la VM (build cassé).

**Correctif proposé.** Générer les tables de types depuis les déclarations des natives (macro ou table unique
`(nom, arité, types, fonction)`) ; regrouper les `RuntimeError` par catégories pour réduire les `match` de
classification ; test « miroir » vérifiant que chaque méthode déclarée dans `types.rs` est acceptée par `dispatch_method`.

---

## 5. 🟠 Typage statique : classes limitées au fichier courant

Les classes importées sont typées `Type::Named(nom)` **sans** informations de classe (`compiler/module_types.rs`
n'exporte que des `Type`). Conséquences : pour une classe importée, pas de vérification statique des surcharges,
du constructeur (arité), des champs typés ni de la visibilité `private` — seule la VM tranche, à l'exécution.
Les modules sont aussi analysés **deux fois** (interface de types puis compilation).

**Correctif proposé.** Exporter un `ClassInfo` (bases, méthodes, champs, membres privés) dans l'interface de module.

---

## 6. 🟠 Tests et intégration continue

* ≈ 70 tests unitaires : surtout `type_checker.rs` (24), `set.rs`, `stdlib/mod.rs`, `resolver.rs`, `types.rs`.
* **Aucun test direct** sur le lexer, le parser, la plupart du compilateur et de la VM (un seul test d'exécution, dans `emit.rs`).
* Le `test_runner` attend un dossier de tests `.ks` avec sorties attendues : **absent de l'archive**.
* Pas de CI (`cargo check`, `cargo test`, `clippy`, `fmt`), pas de test des features `debug_trace` / `profile` / `trace_gc`.
* Les dernières erreurs de build (`Wide`, variantes non traitées) auraient été détectées immédiatement par une CI.

**Correctif proposé.** Un dossier `tests/` de scénarios `.ks` (un par fonctionnalité : surcharges, visibilité, `Set`,
API collections, `Wide`, GC sous charge) + workflow CI minimal.

---

## 7. 🟡 Performance structurelle

| Zone | Constat | Impact |
|---|---|---|
| Globales | `get_global` : `as_string_value()` **clone la chaîne** puis recherche dans un `HashMap<String, Value>` à chaque accès (`println`, appel de fonction…) | allocation par accès |
| Méthodes | `InvokeMethod` clone aussi le nom de la méthode à chaque appel | idem |
| `Dict` / `Set` | `Vec` parcourue linéairement avec `Value::equals` | O(n) par opération, O(n²) à la construction |
| Chaînes | `s = s + x` recrée la chaîne | O(n²) en boucle |
| Boucle VM | `current_position()` (2 lectures) avant **chaque** instruction | coût constant mais inutile hors erreur |
| Positions | deux `Vec<usize>` parallèles au code : ≈ 16 octets par octet de bytecode | mémoire |
| `Value` | `Range { start, stop, step }` gonfle l'énum à ≈ 32 octets | pile et tableaux plus lourds |

**Correctifs proposés.** Résoudre les globales à l'indice (table de slots) ou interner les noms (`Rc<str>`) ;
ne calculer la position qu'en cas d'erreur ; table de hachage pour `Dict`/`Set` (nécessite une notion de hachage cohérente
avec `1 == 1.0`) ; table de lignes compressée (RLE).

---

## 8. 🟡 Désucrage dans le parser

Le parser transforme : `{1, 2}` → `Set(1, 2)` ; les valeurs initiales de champs → méthode cachée `__fields_<Classe>`.
Avantage : VM/compilateur inchangés. Inconvénients : l'AST ne reflète plus le source (messages d'erreur et outils
futurs — formateur, LSP — voient du code synthétique) ; un utilisateur qui redéfinit `Set` casse les littéraux ;
`__fields_X` peut entrer en collision avec un nom utilisateur.

**Correctif proposé.** Nœuds AST dédiés (`Expression::SetLiteral`, champs dans `Statement::Class`) désucrés à la
compilation, avec un nom réservé non écrivable par l'utilisateur.

---

## 9. 🟡 Visibilité `private` : trous de sémantique

* **Espace de champs plat** : `Instance.fields` est un seul `HashMap` ; un champ privé `age` de la classe dérivée écrase
  celui de la base.
* **Visibilité par nom** : toutes les surcharges d'un même nom partagent la visibilité.
* **Contournements** : `json_encode`, `inspect`, `format`, `InvokeBaseMethod` et un `init` privé (`new`) ne passent pas
  par `ensure_member_access`.
* **Fermetures qui s'échappent** : une fonction anonyme créée dans une méthode hérite de `owner_class` et garde l'accès
  aux membres privés même appelée de l'extérieur.

**Correctif proposé.** Clés de champ qualifiées par classe (`Classe::champ`) pour les membres privés ; contrôle dans
les natives d'introspection ; décider explicitement de la règle pour les fermetures.

---

## 10. 🟡 Limites dures et diagnostics

* Littéral tableau / tuple / dict, arguments d'appel : **≤ 255** ; variables locales : ≤ 256 par fonction ;
  méthodes d'une classe : ≤ 255 ; saut : ≤ 65 535 octets (`JumpTooLarge`) — une grosse table de données littérale ou
  une très longue fonction échoue à la compilation.
* `RuntimeError::TypeError` est utilisée **164 fois** : message peu informatif.
* 26 diagnostics de compilation sont créés avec la position `0, 0` ; les erreurs de typage sont localisées à
  l'instruction (`with_location`), pas à l'expression fautive.
* Un piège récurrent : toute instruction est enveloppée dans `Statement::Positioned` ; oublier de la déballer casse
  silencieusement l'analyse (c'était la cause du bug des exports vides dans `analyze_module`).

**Correctifs proposés.** Opérandes larges pour les comptes (comme `Wide` pour les constantes) ou construction
incrémentale (`Array` + `Append`) ; erreurs runtime typées (attendu / reçu) ; positions d'expression dans l'AST du typage.

---

## 11. 🟢 Dette diverse

* Fichiers volumineux : `type_checker.rs` (≈ 2 200 lignes, tests compris), `string.rs` (≈ 1 400), `statements.rs` (≈ 1 400),
  `value.rs` (≈ 1 200) → à découper (résolution de surcharges, visibilité, typage des collections en sous-modules).
* REPL : chaque ligne est vérifiée par un `TypeChecker` neuf (typage quasi inopérant) et `import` n'y fonctionne pas.
* `unsafe` réduit à deux endroits (`transmute` d'`OpCode`, `get_unchecked`), tous deux gardés — acceptable, mais la
  validité du `transmute` repose sur des discriminants contigus (protégés par les tests de `opcode.rs`).
* Pas de bytecode verifier (acceptable tant que seul le compilateur maison produit du bytecode).

---

## Plan d'action suggéré

| Ordre | Chantier | Effort |
|---|---|---|
| 1 | Racines GC (inhibition pendant les rappels natifs + VM de modules sans GC) et test de charge | petit |
| 2 | `MAX_CALL_DEPTH` + garde de cycle à l'affichage / JSON | petit |
| 3 | CI + dossier `tests/` de scénarios `.ks` | moyen |
| 4 | Décision sur le débordement d'entiers | petit (décision) |
| 5 | Table unique des natives et vérification de cohérence des méthodes | moyen |
| 6 | `ClassInfo` exporté entre modules | moyen |
| 7 | Globales par indice / noms internés ; hachage pour `Dict`/`Set` | moyen à gros |
| 8 | Nœuds AST dédiés (fin du désucrage), champs privés qualifiés | moyen |
