# std.statistics

## Statut

`std.statistics` fait partie de la surface officielle de `std`.

## Régression linéaire

```text
export type LinearRegression = { slope: float, intercept: float };

linear_regression(xs: List<float>, ys: List<float>) -> Result<LinearRegression, str>
```

Calcule une régression linéaire simple des valeurs `ys` sur `xs`.

Le résultat contient :

- `slope` : pente de la droite ;
- `intercept` : ordonnée à l'origine.

Exemple :

```text
let result = statistics.linear_regression(
    [1.0, 2.0, 3.0],
    [3.0, 5.0, 7.0]
);

match result {
    Ok(regression) => println(regression.slope);
    Err(error) => println(error);
}
```

Erreurs normales : longueurs différentes, moins de deux points ou variance nulle de `xs`.

Le résultat est un record nommé plutôt qu'un tuple afin de rendre les champs explicites et de rester compatible avec la grammaire de types Kastel.
