// ==================================================================
// Exemple 529 — Analyseur d'URL
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : découper protocole, hôte, port, chemin et paramètres.
// ------------------------------------------------------------------
// Sortie attendue :
//   protocole : https
//   hôte : exemple.fr
//   port : 8080
//   chemin : /chemin/page
//   x = 1
//   y = 2
// ==================================================================

let url = "https://exemple.fr:8080/chemin/page?x=1&y=2";

let p = url.index_of("://");
let protocole = url.slice(0, p);
let reste = url.slice(p + 3, url.size());

let q = reste.index_of("?");
let base = q == -1 ? reste : reste.slice(0, q);
let requete = q == -1 ? "" : reste.slice(q + 1, reste.size());

let s = base.index_of("/");
let hote_port = base.slice(0, s);
let chemin = base.slice(s, base.size());

let d = hote_port.index_of(":");
let hote = hote_port.slice(0, d);
let port = hote_port.slice(d + 1, hote_port.size()).to_int();

println("protocole : " + protocole);
println("hôte : " + hote);
println("port : " + str(port));
println("chemin : " + chemin);
for paire in requete.split("&") {
    let kv = paire.split("=");
    println("{} = {}", kv[0], kv[1]);
}
