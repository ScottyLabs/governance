{ lib, gov }:
let
  inherit (gov) teams;

  mountAccessor = "\${data.vault_auth_backend.oidc.accessor}";

  secretspecPolicy =
    project: profiles:
    lib.concatStringsSep "\n\n" (
      lib.concatMap (profile: [
        "path \"secret/data/secretspec/${project}/${profile}/*\" {\n  capabilities = [\"create\", \"read\", \"update\", \"delete\", \"list\"]\n}"
        "path \"secret/metadata/secretspec/${project}/${profile}/*\" {\n  capabilities = [\"list\", \"read\", \"delete\"]\n}"
      ]) profiles
    );

  projectPolicies = slug: {
    vault_policy = {
      "${slug}_dev" = {
        name = "${slug}-dev";
        policy = secretspecPolicy slug [
          "dev"
          "preview"
        ];
      };
      "${slug}_prod" = {
        name = "${slug}-prod";
        policy = secretspecPolicy slug [
          "prod"
          "staging"
        ];
      };
    };
    vault_identity_group = {
      "${slug}_members" = {
        name = "${slug}-members";
        type = "external";
        policies = [ "${slug}-dev" ];
      };
      "${slug}_admins" = {
        name = "${slug}-admins";
        type = "external";
        policies = [ "${slug}-prod" ];
      };
    };
    vault_identity_group_alias = {
      "${slug}_members" = {
        name = "/projects/${slug}";
        mount_accessor = mountAccessor;
        canonical_id = "\${vault_identity_group.${slug}_members.id}";
      };
      "${slug}_admins" = {
        name = "/projects/${slug}/admins";
        mount_accessor = mountAccessor;
        canonical_id = "\${vault_identity_group.${slug}_admins.id}";
      };
    };
  };

  slugs = builtins.concatMap (
    t:
    if (t.projects or [ ]) == [ ] then
      lib.optional (t.slug != "devops") t.slug
    else
      map (p: p.slug) t.projects
  ) teams;

  merge =
    lib.foldl'
      (
        acc: slug:
        let
          p = projectPolicies slug;
        in
        {
          vault_policy = acc.vault_policy // p.vault_policy;
          vault_identity_group = acc.vault_identity_group // p.vault_identity_group;
          vault_identity_group_alias = acc.vault_identity_group_alias // p.vault_identity_group_alias;
        }
      )
      {
        vault_policy = { };
        vault_identity_group = { };
        vault_identity_group_alias = { };
      }
      slugs;
in
{
  openbao = {
    resource = merge;
  };
}
