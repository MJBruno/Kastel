// Module importé par main.ks (Exporter classes et enums)

export enum Role {
    Admin,
    Invite
}

export class Utilisateur {
    let nom: str;
    let role: Role;

    func initialize(nom: str, role: Role) {
        self.nom = nom;
        self.role = role;
    }

    func description() -> str {
        let etiquette = self.role == Role.Admin ? "admin" : "invité";
        return self.nom + " (" + etiquette + ")";
    }
}
