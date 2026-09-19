// std/json.ks
//
// Encodage/décodage JSON, natif (voir src/stdlib/json.rs — un vrai
// parseur, pas exprimable proprement en Kastel pur).
//
// Usage :
//
//   import std.json;
//   println(json.encode({name: "Ada", age: 36}));
//
//   from std.json import decode;
//   let data = decode("{\"x\": 1, \"y\": [1, 2, 3]}");
//   println(data.get("y"));

export const encode = json_encode;
export const decode = json_decode;

// Pratique pour écrire/lire directement un fichier JSON sans repasser
// par une chaîne intermédiaire à la main.
export func read_file(path) {
    return decode(file_read(path));
}

export func write_file(path, value) {
    file_write(path, encode(value));
}
