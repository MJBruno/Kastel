import export_mod.Personne


let p:Personne = new Personne();
let p1:Personne = new Personne(26);

p.setAge(44)

let v: int = int(p.getAge())

println(v)


let a: int = 45;
let f: float = 16.8;
let s: str = "Hello";
let arrInt: Array <int> =[4,8,3,7,6,2];
let arrFloat: Array <float> =[2.2,4.0,7.6,8.3,6.2];
let dic: Dict<str,int> = {"age":2};
