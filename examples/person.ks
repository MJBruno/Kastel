interface Idisposable {
    func dispose()->Self;
}


export class Person:Idisposable {
    func initialize(){}
    private func dispose() ->Self {
        return this
    }
}