data "keycloak_realm" "this" {
    realm = "scottylabs"
}

resource "keycloak_group" "projects" {
    realm_id = data.keycloak_realm.this.id
    name     = "projects"
}
