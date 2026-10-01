// ==================================================================
// Exemple 520 — Compression RLE : décoder
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : reconstruire le texte (comptes à un chiffre).
// ------------------------------------------------------------------
// Sortie attendue :
//   aaabccdddd
// ==================================================================

func decoder(s: str) -> str {
    let res = "";
    let i = 0;
    while i < s.size() {
        let c = s.char_at(i);
        let n = s.char_at(i + 1).to_int();
        res += c.repeat(n);
        i += 2;
    }
    return res;
}

println(decoder("a3b1c2d4"));
