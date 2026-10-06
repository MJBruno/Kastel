# std.regex

Moteur d'expressions régulières Unicode officiel.

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

Les motifs invalides sont signalés par `Err`. `https` n'est pas concerné par ce module.
