let x = 'global';
{
    let x = 'inner';
    println(x);
}
println(x);
