// std/os.ks
//
// Interaction avec le système/processus. `env`/`cwd`/`clock` sont
// déjà des natives globales (src/stdlib/system.rs) ; `os_name`,
// `os_arch`, `args` et `exit` sont nouvelles (src/stdlib/os.rs).
//
// Pas de `system(...)` (exécution de commande shell arbitraire) —
// exclu volontairement, voir le commentaire en tête de
// src/stdlib/os.rs.
//
// Usage :
//
//   import std.os;
//   println(os.name());
//   println(os.env("HOME"));
//   if os.args().length < 2 {
//       println("usage: kastel script.ks <fichier>");
//       os.exit(1);
//   }

export const name = os_name;
export const arch = os_arch;
export const args = args;
export const exit = exit;
export const env = env;
export const cwd = cwd;
export const clock = clock;
