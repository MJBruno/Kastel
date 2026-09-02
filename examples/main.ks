let r = range(5);

for x in r { 
    println(x); // 0 1 2 3 4
}   
for x in r { 
    println(x); // 0 1 2 3 4  <- rejoué en entier, comme en Python
}   

for x in [10, 20, 30] { 
    println(x); // marche aussi, même bytecode générique
}   