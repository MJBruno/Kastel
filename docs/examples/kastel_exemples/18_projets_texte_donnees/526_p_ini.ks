// ==================================================================
// Exemple 526 — Lecteur de fichier INI
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : sections entre crochets et paires clé=valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   localhost
//   5433
//   demo
// ==================================================================

let texte = "[db]\nhost=localhost\nport=5432\n[app]\nnom=demo";
let config = dict();
let section = "";

for ligne in texte.split("\n") {
    if ligne.starts_with("[") {
        section = ligne.slice(1, ligne.size() - 1);
    } else if ligne.contains("=") {
        let p = ligne.split("=");
        config[section + "." + p[0]] = p[1];
    }
}

println(config["db.host"]);
println(config["db.port"].to_int() + 1);
println(config["app.nom"]);
