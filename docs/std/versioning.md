# Versionnement de std

`std` possède son propre numéro de version, indépendant du numéro du binaire Kastel.

## 1.0.0

La branche 1.x représente l'API stable. Les changements incompatibles nécessitent une nouvelle version majeure.

Les changements non incompatibles peuvent augmenter la version mineure. Les corrections de bugs et de documentation peuvent augmenter la version de correctif.

La compatibilité de la syntaxe et du type checker Kastel reste une contrainte supérieure : une API `std` ne doit pas documenter une forme que le langage ne peut pas parser.
