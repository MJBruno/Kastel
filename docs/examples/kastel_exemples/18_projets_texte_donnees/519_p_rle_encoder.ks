// ==================================================================
// Exemple 519 — Compression RLE : encoder
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : remplacer chaque suite de lettres identiques par lettre + nombre.
// ------------------------------------------------------------------
// Sortie attendue :
//   a3b1c2d4
//   a1b1c1
// ==================================================================

func encoder(s: str) -> str {
    let res = "";
    let i = 0;
    while i < s.size() {
        let c = s.char_at(i);
        let n = 1;
        while i + n < s.size() && s.char_at(i + n) == c {
            n += 1;
        }
        res += c + str(n);
        i += n;
    }
    return res;
}

println(encoder("aaabccdddd"));
println(encoder("abc"));
