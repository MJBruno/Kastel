// std/path.ks
//
// Manipulation de chemins, natif pour tout ce qui touche à la
// normalisation/aux séparateurs OS (voir src/stdlib/path.rs), pur
// Kastel pour le reste.
//
// Usage :
//
//   import std.path;
//   let full = path.join(["dossier", "sous-dossier", "fichier.txt"]);
//   println(path.extension(full));    // txt
//   println(path.basename(full));     // fichier.txt

export const join = path_join;
export const exists = path_exists;
export const is_dir = path_is_dir;
export const is_file = path_is_file;
export const absolute = path_absolute;
export const basename = path_basename;
export const dirname = path_dirname;
export const extension = path_extension;
export const stem = path_stem;

// join() prend un tableau (pas de varargs en Kastel) — raccourci
// pour joindre juste deux segments sans construire le tableau soi-même.
export func join2(a, b) {
    return join([a, b]);
}

export func has_extension(filepath, wanted) {
    return extension(filepath) == wanted;
}
