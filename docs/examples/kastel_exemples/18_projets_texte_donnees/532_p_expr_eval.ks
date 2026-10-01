// ==================================================================
// Exemple 532 — Évaluateur d'expressions arithmétiques
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : analyse descendante récursive avec priorités (* avant +) et parenthèses.
// ------------------------------------------------------------------
// Sortie attendue :
//   11
//   21
//   80
//   1
// ==================================================================

let jetons = [];
let pos = 0;

func tokeniser(s: str) -> List<str> {
    let res = [];
    let i = 0;
    while i < s.size() {
        let c = s.char_at(i);
        if c.is_digit() {
            let j = i;
            while j < s.size() && s.char_at(j).is_digit() { j += 1; }
            res.add(s.slice(i, j));
            i = j;
        } else {
            if c != " " { res.add(c); }
            i += 1;
        }
    }
    return res;
}

// expr := terme (('+' | '-') terme)*
func expr() -> int {
    let v = terme();
    while pos < jetons.size() && (jetons[pos] == "+" || jetons[pos] == "-") {
        let op = jetons[pos];
        pos += 1;
        let w = terme();
        v = op == "+" ? v + w : v - w;
    }
    return v;
}

// terme := facteur (('*' | '/') facteur)*
func terme() -> int {
    let v = facteur();
    while pos < jetons.size() && (jetons[pos] == "*" || jetons[pos] == "/") {
        let op = jetons[pos];
        pos += 1;
        let w = facteur();
        v = op == "*" ? v * w : idiv(v, w);
    }
    return v;
}

// facteur := nombre | '(' expr ')'
func facteur() -> int {
    let t = jetons[pos];
    pos += 1;
    if t == "(" {
        let v = expr();
        pos += 1;          // saute la parenthèse fermante
        return v;
    }
    return t.to_int();
}

func evaluer(s: str) -> int {
    jetons = tokeniser(s);
    pos = 0;
    return expr();
}

println(evaluer("2+3*(4-1)"));
println(evaluer("(1+2)*(3+4)"));
println(evaluer("100-4*5"));
println(evaluer("20/4/5"));
