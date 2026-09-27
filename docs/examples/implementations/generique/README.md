# Kastel — tests `.ks` pour les génériques

## Tests valides

Le contenu de `valid/` doit être accepté et exécuté avec un code de sortie `0`.

Commande :

```powershell
kastel .\valid\01_generic_function.ks
```

## Tests invalides

Le contenu de `invalid/` doit être refusé par le parser ou le type-checker avec un code de sortie non nul.

Commande :

```powershell
kastel .\invalid\01_wrong_function_arity.ks
```

## Test protected

`valid/07_generic_inheritance_protected.ks` contient une ligne volontairement commentée dans la section concernée. Pour vérifier l'interdiction externe de `protected`, utiliser `invalid/10_protected_external_access.ks`.

Les tests couvrent :

- fonctions génériques ;
- inférence ;
- appels génériques explicites ;
- classes génériques ;
- méthodes génériques ;
- méthode statique générique ;
- classes génériques + méthodes génériques ;
- alias de type génériques ;
- héritage générique ;
- `protected` avec génériques ;
- interfaces génériques ;
- enums génériques ;
- types génériques imbriqués ;
- ambiguïté `f<T>(...)` / opérateur `<` ;
- `dynamic` ;
- erreurs d'arité ;
- incompatibilité de types ;
- mauvais paramétrage d'une classe/interface ;
- paramètre générique inconnu ;
- paramètres génériques dupliqués.
