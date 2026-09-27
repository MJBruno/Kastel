try {
    let x: int = 12
} catch (e: Err) {
    println(e.kind);
    println(e.message);
}