// ====================================================================
// Kastel — match sur Option et Result imbriqués
// Notions : Ok(Some(x)), Ok(None), Err(e)
// Résultat attendu :
//   valeur : 8
//   pas de valeur
//   erreur : réseau
// ====================================================================

func traiter(r) {
    match r {
        Ok(Some(x)) => { println("valeur : {}", x); },
        Ok(None) => { println("pas de valeur"); },
        Err(e) => { println("erreur : {}", e); }
    }
}

traiter(Ok(Some(8)));
traiter(Ok(None));
traiter(Err("réseau"));
