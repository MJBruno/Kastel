# std.regex

Moteur d'expressions régulières Unicode officiel de Kastel.

Le moteur est implémenté directement dans le runtime Kastel/Rust et **ne dépend d'aucune bibliothèque externe**.

## API

```text
is_match(pattern: str, text: str) -> Result<bool, str>
find(pattern: str, text: str) -> Result<Option<RegexMatch>, str>
find_all(pattern: str, text: str) -> Result<List<RegexMatch>, str>
replace(pattern: str, text: str, replacement: str) -> Result<str, str>
split(pattern: str, text: str) -> Result<List<str>, str>
escape(text: str) -> str
```

`RegexMatch.start` et `end` sont des indices de caractères Unicode, avec `end` exclusif. `groups` contient les groupes capturants 1..N.

## Syntaxe supportée

- caractères littéraux ;
- `.` ;
- ancres `^` et `$` (`\A` et `\z` sont des alias) ;
- classes `[abc]`, `[^abc]`, plages `[a-z]` ;
- classes `\d`, `\D`, `\w`, `\W`, `\s`, `\S` ;
- échappements `\n`, `\r`, `\t`, `\f`, `\v`, `\0` ;
- groupes capturants `( ... )` ;
- alternance `a|b` ;
- quantificateurs `*`, `+`, `?`, `{m}`, `{m,n}`, `{m,}` ;
- quantificateurs non-gourmands `*?`, `+?`, `??`, `{m,n}?` ;
- frontière de mot `\b` et non-frontière `\B` ;
- remplacements `$0`, `$1`, `$2`, ... et `$$`.

Les lookarounds, groupes non capturants `(?:...)`, rétro-références et classes de propriétés Unicode `\p{...}` ne font pas partie de l'API 1.0. Ils sont refusés explicitement au lieu d'être interprétés différemment.

Les motifs invalides, les groupes de remplacement inexistants et les limites d'exécution sont retournés par `Result<_, str>` et ne doivent jamais provoquer de panique.
