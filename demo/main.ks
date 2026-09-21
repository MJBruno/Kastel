type Personne = {
    name:str
}

let p:Personne = {name:"Bruno"}
let d:Personne = {"name":"Bruno"}

println(type(p))
println(type(d))