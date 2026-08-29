# Kastel

**Kastel** est un langage de programmation moderne, dynamique et interprété, conçu pour offrir une syntaxe simple et expressive tout en reposant sur une architecture d’exécution basée sur une **machine virtuelle (VM)**.

Inspiré de **JavaScript** et de **Python**, Kastel reprend certains de leurs principes les plus accessibles — syntaxe concise, programmation dynamique et facilité d’utilisation — tout en développant sa propre architecture interne orientée vers l’exécution par bytecode.

## Une syntaxe simple et expressive

Kastel est conçu pour permettre d'écrire du code lisible avec un minimum de complexité syntaxique. Son inspiration de JavaScript et Python se retrouve notamment dans la déclaration des variables, les fonctions, les structures de contrôle et la manipulation des objets.

Exemple :

```JavaScript
let name = "Kastel";
let version = 1;

function greet(name) {
    return "Hello, " + name;
}

println(greet(name));
```

L'objectif est de conserver une expérience de programmation simple tout en fournissant une base suffisamment solide pour construire des programmes plus complexes.

## Typage dynamique

Kastel utilise un **système de typage dynamique**.

Le type d'une valeur est déterminé pendant l'exécution plutôt qu'imposé lors de la compilation du programme. Une variable peut donc contenir différentes valeurs au cours de son cycle de vie.

```JavaScript
let value = 42;
value = "Kastel";
value = true;
```

Cette approche permet de développer rapidement et rend le langage particulièrement adapté au scripting, au prototypage et à la création de programmes nécessitant une grande flexibilité.

## Exécution par machine virtuelle

Contrairement à un langage compilé directement en code machine, Kastel est exécuté par une **machine virtuelle**.

Le programme source suit généralement cette chaîne d'exécution :

```text
Code Kastel
    │
    ▼
Lexer
    │
    ▼
Parser
    │
    ▼
AST
    │
    ▼
Compilateur
    │
    ▼
Bytecode
    │
    ▼
Machine virtuelle
    │
    ▼
Exécution
```

Le compilateur transforme le code Kastel en **bytecode**, puis la machine virtuelle interprète et exécute ces instructions.

Cette architecture sépare clairement le langage de la machine sur laquelle il s'exécute et constitue une base permettant d'ajouter ultérieurement différentes optimisations et fonctionnalités d'exécution.

## Une architecture conçue pour évoluer

Kastel n'est pas uniquement conçu comme un langage de script. Son architecture est pensée pour évoluer progressivement autour de plusieurs composants :

- **Lexer** pour transformer le code source en tokens ;
- **Parser** pour construire la représentation syntaxique du programme ;
- **Compilateur** pour produire le bytecode ;
- **Pool de constantes** pour gérer les valeurs constantes du programme ;
- **Bytecode** représentant les instructions exécutables ;
- **Machine virtuelle** chargée d'exécuter les instructions ;
- **Modules** permettant d'organiser et de réutiliser le code ;
- **Structures de données dynamiques** ;
- **Système d'erreurs** permettant de signaler clairement les problèmes rencontrés pendant la compilation ou l'exécution.

Cette séparation des responsabilités permet de faire évoluer chaque partie du langage indépendamment tout en conservant une architecture cohérente.

## Inspiré de JavaScript et Python, mais indépendant

Kastel s'inspire de JavaScript et Python sans chercher à reproduire leurs implémentations.

De JavaScript, Kastel reprend notamment l'idée d'un langage dynamique, flexible et adapté à une exécution interprétée.

De Python, Kastel reprend une philosophie mettant l'accent sur la lisibilité, la simplicité et la productivité du développeur.

Cependant, **Kastel possède sa propre architecture, son propre bytecode, sa propre machine virtuelle et ses propres choix de conception**.

## Objectif du projet

L'objectif de Kastel est de construire un langage qui combine :

**simplicité + flexibilité + architecture de machine virtuelle + extensibilité.**

Le projet sert également de base pour explorer concrètement la conception d'un langage de programmation moderne : analyse lexicale, parsing, compilation, génération de bytecode, conception d'une VM, gestion de la mémoire, appels de fonctions, modules et évolution du runtime.

Kastel est donc à la fois un langage utilisable et un projet d'expérimentation autour de la **conception des langages de programmation et des machines virtuelles**.

## Vision

La vision de Kastel est de construire progressivement un environnement d'exécution complet, cohérent et performant autour d'un langage dynamique moderne.

L'architecture par bytecode et machine virtuelle constitue le cœur du projet. Elle offre une fondation permettant d'introduire progressivement de nouvelles fonctionnalités sans remettre en cause les principes fondamentaux du langage.

**Kastel : un langage dynamique, simple et moderne, exécuté par sa propre machine virtuelle.**