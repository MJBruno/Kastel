// std/file.ks
//
// Entrées/sorties fichier, natif (voir src/stdlib/file.rs — appels
// système, impossibles en Kastel pur).
//
// Usage :
//
//   import std.file;
//   file.write("out.txt", "hello");
//   println(file.read("out.txt"));
//
//   from std.file import read_lines;
//   for line in read_lines("data.csv") {
//       println(line);
//   }

export const read = file_read;
export const read_lines = file_read_lines;
export const write = file_write;
export const append = file_append;
export const exists = file_exists;
export const delete = file_delete;
export const size = file_size;
