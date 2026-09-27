// ============================================
// Option<T>
// ============================================

func find_user(id: int) -> Option<str> {
    if id == 1 {
        return Some("Bruno");
    }

    return None;
}


// ============================================
// Result<T, E>
// ============================================

func load_user(id: int) -> Result<str, str> {
    let user = find_user(id).ok_or("Utilisateur introuvable");

    return user;
}


// ============================================
// ? avec Result
// ============================================

func get_user_score(id: int) -> Result<int, str> {
    let user = load_user(id)?;

    if user == "Bruno" {
        return Ok(100);
    }

    return Err("Utilisateur inconnu");
}


// ============================================
// Plusieurs propagations avec ?
// ============================================

func process_user(id: int) -> Result<int, str> {
    let score = get_user_score(id)?;

    let bonus = 20;

    return Ok(score + bonus);
}


// ============================================
// Exception runtime
// ============================================

func dangerous_operation() -> float {
    // Provoque une exception runtime
    return 10 / 0;
}


// ============================================
// PROGRAMME
// ============================================

// ---------- OPTION ----------

let user = find_user(1);

println("Option:");
println(user.to_string());

let missing = find_user(999);

println(missing.to_string());


// ---------- RESULT ----------

let result = load_user(1);

println("Result:");
println(result.to_string());

let error = load_user(999);

println(error.to_string());


// ---------- RESULT + ? ----------

let processed = process_user(1);

println("Processed:");
println(processed.to_string());


// ---------- ERREUR RESULT ----------

let failed = process_user(999);

println("Failed:");
println(failed.to_string());


// ---------- TRY / CATCH ----------

println("Try/Catch:");

try {
    let value = dangerous_operation();

    println(value);
}
catch (e: Err) {
    println("Exception capturee:");
    println(e.to_string());
}