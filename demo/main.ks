let username = None;

let result = username.ok_or("Utilisateur introuvable");

println(result.to_string());