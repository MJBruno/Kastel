// ==================================================================
// Exemple 447 — Arguments de la ligne de commande
// Catégorie : Fichiers, JSON, système
// ------------------------------------------------------------------
// args() renvoie la liste des arguments passés au programme.
// ==================================================================

let a = args();
println(a.size() >= 0);
for x in a {
    println("argument : " + x);
}
