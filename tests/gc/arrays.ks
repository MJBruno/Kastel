// GC - arrays
let keep = [1, 2, 3];

for i in range(10000) {
    let temp = [i, i + 1, i + 2, "temporary"];
}

println(keep);


// keep ───────────────→ [1, 2, 3]
//                          ↑
//                      doit rester vivant

// temp → [i, i+1, i+2, "temporary"]
//         ↑
//      temporaire
//         ↓
//    doit être récupérable