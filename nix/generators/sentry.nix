{ gov }:
let
  inherit (gov) teams;

  org = "\${var.sentry_organization}";

  reposOf = t: (t.repos or [ ]) ++ builtins.concatMap (p: p.repos or [ ]) (t.projects or [ ]);

  hasSentry = r: (r.features or { }) ? sentry;

  perTeam = map (
    t:
    let
      inherit (t) slug;
      repos = builtins.filter hasSentry (reposOf t);
    in
    {
      inherit slug repos;
      inherit (t) name;
    }
  ) teams;

  activeTeams = builtins.filter (t: t.repos != [ ]) perTeam;

  keyOf = slug: repo: "${slug}_${builtins.replaceStrings [ "-" ] [ "_" ] repo.name}";

  teamEntries = map (t: {
    name = t.slug;
    value = {
      organization = org;
      inherit (t) name;
      inherit (t) slug;
    };
  }) activeTeams;

  projectEntries = builtins.concatMap (
    t:
    map (
      repo:
      let
        key = keyOf t.slug repo;
        platform = repo.features.sentry.platform or null;
        base = {
          organization = org;
          teams = [ "\${sentry_team.${t.slug}.slug}" ];
          inherit (repo) name;
          slug = repo.name;
        };
      in
      {
        name = key;
        value =
          base
          // (
            if platform != null then
              { inherit platform; }
            else
              {
                lifecycle = {
                  ignore_changes = [ "platform" ];
                };
              }
          );
      }
    ) t.repos
  ) activeTeams;

  keyEntries = builtins.concatMap (
    t:
    map (
      repo:
      let
        key = keyOf t.slug repo;
      in
      {
        name = key;
        value = {
          organization = org;
          project = "\${sentry_project.${key}.slug}";
          name = "default";
        };
      }
    ) t.repos
  ) activeTeams;

  vaultEntries = builtins.concatMap (
    t:
    map (
      repo:
      let
        key = keyOf t.slug repo;
      in
      {
        name = "${key}_sentry_dsn";
        value = {
          mount = "secret";
          name = "secretspec/${repo.name}/prod/SENTRY_DSN";
          data_json = "\${jsonencode({ value = sentry_key.${key}.dsn })}";
        };
      }
    ) t.repos
  ) activeTeams;
in
{
  sentry = {
    resource = {
      sentry_team = builtins.listToAttrs teamEntries;
      sentry_project = builtins.listToAttrs projectEntries;
      sentry_key = builtins.listToAttrs keyEntries;
      vault_kv_secret_v2 = builtins.listToAttrs vaultEntries;
    };
  };
}
