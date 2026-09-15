try {
    throw 'boom';
} catch (e) {
    println(e);
} finally {
    println('finally');
}
