// ==================================================================
// Exemple 361 — Pipeline de tâches
// Catégorie : async / await
// ------------------------------------------------------------------
// Le résultat d'une étape alimente la suivante.
// ------------------------------------------------------------------
// Sortie attendue :
//   30
// ==================================================================

async func etape1() -> int { return 10; }
async func etape2(x: int) -> int { return x + 5; }
async func etape3(x: int) -> int { return x * 2; }

let a = await etape1();
let b = await etape2(a);
let c = await etape3(b);
println(c);
