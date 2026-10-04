// ====================================================================
// Kastel — Exporter classes et enums
// Notions : export class, export enum, export type
// Résultat attendu :
//   Ana (admin)
//   true
// ====================================================================

from comptes import { Utilisateur, Role };

let u = new Utilisateur("Ana", Role.Admin);
println(u.description());
println(u.role == Role.Admin);
