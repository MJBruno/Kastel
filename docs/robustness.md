# Robustesse : GC et limites de profondeur

Ce document décrit les correctifs des deux points critiques de l'audit d'architecture.

## 1. GC : racines complètes

Rappel : le GC est un *mark & sweep* posé sur des `Rc`. Le « sweep » ne libère pas, il **vide** (`break_cycle`) tout objet
non marqué. La correction dépend donc d'un ensemble de racines exact.

| Trou d'origine | Correctif |
|---|---|
| Résultats intermédiaires de `map`/`filter`/`reduce`/`any`/`all` tenus par une variable Rust pendant les rappels | `VirtualMachine::temp_roots` (racines temporaires, marquées par le GC) ; `with_temp_roots` enracine le receveur et les arguments, `protect` enracine chaque résultat |
| Cliché des éléments d'un tableau modifié par le rappel | le cliché est enraciné |
| Méthodes d'itérateur (`next`, `to_list`, `any`…) | `invoke_iterator_method` enracine ses arguments ; `iterator_collect` protège chaque valeur |
| `new` : classe et arguments retirés de la pile pendant les initialiseurs de champs et le constructeur | `op_new_instance` les enracine |
| `import` : le module s'exécute dans une VM imbriquée dont les racines ne voyaient pas le programme appelant | `VirtualMachine::pin_roots()` épingle pile, globales, frames, upvalues et racines temporaires de la VM appelante (garde RAII, pile LIFO dans `runtime::gc`) |
| Marquage récursif : une structure imbriquée sur des dizaines de milliers de niveaux faisait déborder la pile | marquage **itératif** (file `pending`) |

Règle pour le futur code natif : *toute valeur qui n'est plus sur la pile VM et qui survit à un appel `invoke_sync`
doit être enracinée* (`with_temp_roots` / `protect`).

## 2. Limites de profondeur

| Zone | Limite | Erreur |
|---|---|---|
| Appels Kastel (frames) | `MAX_CALL_DEPTH` = 100 000 | `RuntimeError::StackOverflow` (catchable) |
| Rappels natifs imbriqués (`map`, constructeurs, itérateurs…) | `MAX_NATIVE_DEPTH` = 500 | `RuntimeError::StackOverflow` |
| Imbrication syntaxique (parenthèses, blocs, unaires) | `MAX_NESTING_DEPTH` = 500 (parser) | erreur de parsing |
| Profondeur d'expression (compilateur et vérificateur de types) | `MAX_EXPRESSION_DEPTH` = 5 000 | `CompileError::ExpressionTooDeep` |
| Affichage de conteneurs imbriqués | `MAX_DISPLAY_DEPTH` = 100 ; cycle détecté | `...` (ex. `[...]`) |
| `json_encode` | `MAX_JSON_DEPTH` = 512 ; cycle détecté | `RuntimeError::CyclicStructure` |
| `json_decode` | `MAX_JSON_DEPTH` = 512 | erreur `json.decode` |

Autres mesures :

* **Thread à pile large** : `main.rs` exécute tout (REPL compris) dans un thread de 256 Mo. Les limites ci-dessus sont
  dimensionnées pour y tenir largement, y compris en build debug.
* **Bytecode partagé** : `Function.chunk` est un `Rc<Chunk>` ; chaque appel ne copie plus le fragment (code, positions,
  constantes) — sans quoi 100 000 frames auraient consommé des gigaoctets.

## Limites restantes

* Le destructeur d'`Rc` reste récursif : libérer une structure imbriquée sur des centaines de milliers de niveaux
  consomme de la pile (couvert par les 256 Mo, mais pas illimité).
* Le GC ne collecte pas *pendant* un appel natif qui ne rappelle pas de code Kastel (aucun risque, aucune collecte).

## Tests

`src/vm/machine/robustness_tests.rs` : résultats de `map` intacts après GC, arguments de constructeur intacts,
`import` sans corruption du programme appelant, marquage d'une structure de 100 000 niveaux, récursion infinie
signalée, récursion légitime de 5 000 niveaux, rappels natifs bornés, source trop imbriquée refusée, chaîne
d'opérateurs de 6 000 termes refusée, affichage et `json_encode` d'un tableau cyclique.
