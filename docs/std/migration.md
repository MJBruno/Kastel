# Migration vers std 1.0

`std 1.0` privilégie une surface unique et stable.

- `std.strings` → `std.string`
- `std.file` → `std.fs`
- `std.statistic` → `std.statistics`
- `Array` → `List`
- `length()` → `size()`
- `push()` → `add()`
- `has()` → `contains()`
- les opérations de chemin sont dans `std.path`
- les opérations JSON sont dans `std.json`
- les constructeurs réseau renvoient `Result`
- le client HTTP renvoie `Result<HttpResponse, str>`
