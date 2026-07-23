{ lib, gov }:
let
  inherit (gov) teams;

  reposOf = t: (t.repos or [ ]) ++ builtins.concatMap (p: p.repos or [ ]) (t.projects or [ ]);

  hasPosthog = r: (r ? features) && (r.features ? posthog);

  entries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
    in
    builtins.concatMap (
      r:
      let
        key = "${slug}_${builtins.replaceStrings [ "-" ] [ "_" ] r.name}";
      in
      [
        {
          type = "posthog_project";
          name = key;
          body = {
            inherit (r) name;
          };
        }
        {
          type = "vault_kv_secret_v2";
          name = "${key}_posthog_key";
          body = {
            mount = "secret";
            name = "secretspec/${r.name}/prod/POSTHOG_KEY";
            data_json = "\${jsonencode({ value = posthog_project.${key}.api_token })}";
          };
        }
        {
          type = "vault_kv_secret_v2";
          name = "${key}_posthog_host";
          body = {
            mount = "secret";
            name = "secretspec/${r.name}/prod/POSTHOG_HOST";
            data_json = "\${jsonencode({ value = var.posthog_host })}";
          };
        }
      ]
    ) (builtins.filter hasPosthog (reposOf t))
  ) teams;

  byType = lib.groupBy (e: e.type) entries;
  resource = builtins.mapAttrs (
    _: es:
    builtins.listToAttrs (
      map (e: {
        inherit (e) name;
        value = e.body;
      }) es
    )
  ) byType;
in
{
  posthog = { inherit resource; };
}
