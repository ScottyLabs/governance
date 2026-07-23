{ lib, gov }:
let
  inherit (gov) teams groupsOf;

  underscore = s: builtins.replaceStrings [ "-" ] [ "_" ] s;

  gatewayReposOf =
    t:
    builtins.filter (r: (r.features or { }) ? ai_gateway) (
      builtins.concatMap (g: g.repos or [ ]) (groupsOf t)
    );

  envParams = g: [
    {
      env = "prod";
      budget = g.prod_monthly_budget or 20.0;
      rpm = g.prod_rpm_limit or 1000;
      tpm = g.prod_tpm_limit or 4000000;
      parallel = g.prod_max_parallel_requests or 100;
    }
    {
      env = "dev";
      budget = g.dev_monthly_budget or 5.0;
      rpm = g.dev_rpm_limit or 200;
      tpm = g.dev_tpm_limit or 1000000;
      parallel = g.dev_max_parallel_requests or 20;
    }
  ];

  teamEntries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
      teamKey = underscore slug;
    in
    lib.optionals (gatewayReposOf t != [ ]) [
      {
        name = teamKey;
        value = {
          team_alias = slug;
          blocked = false;
          team_member_permissions = [ ];
        };
      }
    ]
  ) teams;

  keyEntries = builtins.concatMap (
    t:
    let
      teamKey = underscore t.slug;
    in
    builtins.concatMap (
      repo:
      let
        key = underscore repo.name;
        gateway = repo.features.ai_gateway;
      in
      map (p: {
        name = "${key}_${p.env}";
        value = {
          key_alias = "${repo.name}-${p.env}";
          team_id = "\${litellm_team.${teamKey}.id}";
          max_budget = p.budget;
          budget_duration = "monthly";
          rpm_limit = p.rpm;
          tpm_limit = p.tpm;
          max_parallel_requests = p.parallel;
          metadata = {
            project = repo.name;
          };
        };
      }) (envParams gateway)
    ) (gatewayReposOf t)
  ) teams;

  secretEntries = builtins.concatMap (
    t:
    builtins.concatMap (
      repo:
      let
        key = underscore repo.name;
      in
      builtins.concatMap
        (
          profile:
          let
            source = if profile == "prod" then "prod" else "dev";
          in
          [
            {
              name = "${key}_${profile}_litellm_api_key";
              value = {
                mount = "secret";
                name = "secretspec/${repo.name}/${profile}/LITELLM_API_KEY";
                data_json = "\${jsonencode({ value = litellm_key.${key}_${source}.key })}";
              };
            }
            {
              name = "${key}_${profile}_litellm_base_url";
              value = {
                mount = "secret";
                name = "secretspec/${repo.name}/${profile}/LITELLM_BASE_URL";
                data_json = "\${jsonencode({ value = var.litellm_url })}";
              };
            }
          ]
        )
        [
          "prod"
          "staging"
          "preview"
          "dev"
        ]
    ) (gatewayReposOf t)
  ) teams;

  resource =
    (lib.optionalAttrs (teamEntries != [ ]) { litellm_team = builtins.listToAttrs teamEntries; })
    // (lib.optionalAttrs (keyEntries != [ ]) { litellm_key = builtins.listToAttrs keyEntries; })
    // (lib.optionalAttrs (secretEntries != [ ]) {
      vault_kv_secret_v2 = builtins.listToAttrs secretEntries;
    });
in
{
  litellm = lib.optionalAttrs (resource != { }) { inherit resource; };
}
