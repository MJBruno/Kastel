# Surcharge de fonctions, classes importées, REPL et constructeur privé

Ce document couvre les « limites connues » levées dans cette version.

## 1. Fonctions globales surchargées par arité

```kastel
func area(side: int) -> int { return side * side; }
func area(width: int, height: int) -> int { return width * height; }

area(3);      // 9
area(2, 5);   // 10
let f = area; // se passe comme une valeur : f(4), f(4, 6)
```

* La surcharge se fait par **nombre de paramètres**. Même nom **et** même arité = erreur (`DuplicateFunction`).
* À la **compilation**, l'appel direct choisit la signature par arité et par type (comme pour une méthode) ; un nombre d'arguments
  sans surcharge, ou un type incompatible, est refusé.
* À l'**exécution**, les surcharges forment un objet unique (`Object::Overloads`, `OpCode::Overload`) : la première déclaration
  définit la globale, les suivantes s'y ajoutent **en place**. Un appel choisit la fermeture dont l'arité égale le nombre
  d'arguments — y compris quand la fonction est passée en rappel (`map(area)`), stockée dans une variable ou importée.
* **Modules** : plusieurs `export func f` de même nom forment **un seul export**. `import m; m.f(1)` et `from m import f` fonctionnent.
  Côté vérificateur, une fonction surchargée importée est **dynamique** (les appels ne sont pas vérifiés statiquement).
* **Limites** : seules les fonctions globales se surchargent (pas celles déclarées dans une autre fonction) ; dans le REPL, une
  fonction ne peut pas être redéfinie.

## 2. Classes importées vérifiées comme des classes locales

L'interface de types d'un module contient maintenant, pour chaque classe et interface : bases, méthodes surchargées (avec types),
champs typés et membres privés. Une classe importée (`import m.C;`, `from m import C [as D];`, `from m import *;`) est donc vérifiée
**à la compilation** :

* arité et types des constructeurs surchargés, constructeur par défaut implicite ;
* surcharges de méthodes, y compris héritées ;
* types des champs ;
* visibilité `private` (`p.age` refusé hors de la classe).

Un alias d'import (`from m import Personne as P`) désigne la même classe. Les bases d'une classe importée sont importées avec elle
(sous leur nom d'origine). La VM contrôle toujours la visibilité à l'exécution.

## 3. `import` dans le REPL

Le REPL résout les imports à partir du **répertoire courant** (comme un fichier placé là) :

```text
>>> import std.math;
>>> math.gcd(48, 18)
6
>>> import std.math.Complexe;
>>> new Complexe(3, 4).magnitude()
5.0
```

Le cache des modules est partagé entre les lignes. Chaque ligne est vérifiée seule : les types des lignes précédentes ne sont pas
mémorisés.

## 4. Constructeur `private`

`private func initialize(...)` interdit `new` **hors du corps de la classe** qui le déclare — à la compilation quand la classe est
connue (locale ou importée), et à l'exécution sinon. Une classe dérivée qui déclare son propre `initialize` public peut déléguer au
constructeur privé de sa base par `base.initialize(...)` :

```kastel
class Base    { private func initialize() { this.v = 7; } }
class Derived: Base { func initialize() { base.initialize(); } }

new Derived();  // permis
new Base();     // refusé
```

Sans constructeur déclaré (constructeur par défaut implicite), aucun contrôle.

### Précision sur `json_encode` et les champs privés

Une affirmation antérieure (« `json_encode` lit les champs privés directement ») était **inexacte** : `json_encode` n'encode pas les
instances (erreur), et `str` / `inspect` / `println` d'une instance affichent seulement `<Classe instance>`. Aucune fonction native ne
lit les champs d'une instance. Un test de non-régression le garantit (`json_and_display_never_expose_private_fields`).
