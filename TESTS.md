# Suite de tests Kastel

Cette suite couvre les fonctionnalités visibles dans le source actuel de Kastel : lexer/parser, valeurs primitives, nombres, chaînes, tableaux, dictionnaires, tuples, objets dynamiques, variables constantes/mutables, affectations composées, fonctions, récursion, closures, fonctions fléchées, contrôle de flux, `for ... in`, `while`, `break`, `continue`, `range`, itérateurs, `match`, gardes, exceptions, `finally`, classes, héritage, `base`, interfaces, `is`, imports/exports, natives de conversion, mathématiques, formatage et le chemin GC des itérateurs.

## Arborescence

```text
test/
├── errors/           # chaque fichier doit produire une erreur Kastel
└── regression/       # chaque fichier possède un .expected

scripts/
├── run_tests.ps1     # Windows / PowerShell
└── run_tests.sh      # Linux / macOS / Git Bash
```

## Exécution Windows

Depuis la racine du projet :

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run_tests.ps1
```

ou, avec PowerShell 7 :

```powershell
pwsh -File .\scripts\run_tests.ps1
```

## Exécution Unix

```bash
./scripts/run_tests.sh
```

## Test manuel direct

```powershell
cargo run -q --bin kastel test/regression/18_functions.ks
```

## Mise à jour des sorties

Les sorties `.expected` sont volontairement versionnées. Pour modifier le comportement attendu d'un test, exécuter le fichier, vérifier la sortie, puis modifier le `.expected` manuellement. Ne jamais accepter automatiquement une sortie simplement parce qu'elle change : une différence peut signaler une régression.

## Cas volontairement non comparés à une valeur fixe

`clock()`, `cwd()`, `env()`, `rand()`, `rand_int()` et `rand_range()` sont testés uniquement sur des propriétés déterministes comme leur type. Leur valeur concrète dépend de l'environnement ou du hasard.

`input()` n'est pas automatisé dans cette suite, car il nécessite une entrée interactive. Il doit être testé manuellement.

## Classes et interfaces

La syntaxe testée est celle du parser actuel :

```kastel
class Dog: Animal {
    func speak() {
        return base.speak();
    }
}

interface Speakable {
    func speak();
}
```

Les classes utilisent `init(...)` comme constructeur lors de `new Class(...)`.

## Match

Les patterns testés sont :

```kastel
1
1 | 2 | 3
1 .. 10
1 ..= 10
[x, y]
_
x if condition
```

## Imports

La suite couvre :

```kastel
import module;
from module import name;
from module import name as alias;
from module import *;
```

Les modules de test sont placés dans `test/regression/` pour respecter la résolution relative actuelle du `ModuleLoader`.

## Important : `and` / `or` / `not`

Le lexer actuel expose les opérateurs logiques sous les formes symboliques :

```kastel
&&
||
!
```

Les mots `and`, `or` et `not` ne font pas partie des mots-clés actuellement définis par `Token::keyword`. La suite n'utilise donc pas cette syntaxe.

## Important : ranges

La syntaxe inclusive réellement lexée par le source actuel est :

```kastel
1 ..= 10
```

et non `1 ... 10`.

## Important : test runner historique

Le binaire `kastel` affiche actuellement une ligne de chronométrage `Process finished...`. Le script de cette suite retire cette ligne avant comparaison afin que les timings ne rendent pas les `.expected` non déterministes.

## Statut de cette livraison

Nombre de tests source :

- 56 tests de régression déterministes/propriété
- 30 tests d'erreur
- 2 modules auxiliaires

Cette suite a été construite à partir du code source fourni. L'environnement de génération ne dispose pas de `cargo`/`rustc`, donc l'exécution réelle de la suite n'a pas pu être effectuée ici. Une validation complète nécessite de lancer le script sur la machine possédant le toolchain Rust du projet.
