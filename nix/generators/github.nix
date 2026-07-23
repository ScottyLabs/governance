{ lib, gov }:
let
  inherit (gov) teams;

  hasGithub = gov.org ? github;

  forgejoUrl =
    if (gov.org ? forgejo) then
      (gov.org.forgejo.url or "https://codeberg.org")
    else
      "https://codeberg.org";
  forgejoOrg = if (gov.org ? forgejo) then gov.org.forgejo.org else "";

  underscore = s: builtins.replaceStrings [ "-" ] [ "_" ] s;

  teamRepos =
    t:
    map (r: { inherit t r; }) (
      (t.repos or [ ]) ++ builtins.concatMap (p: p.repos or [ ]) (t.projects or [ ])
    );
  allRepos = builtins.concatMap teamRepos teams;

  keyOf = e: "${e.t.slug}_${underscore e.r.name}";

  repoResources = map (e: rec {
    key = keyOf e;
    codebergUrl = "${forgejoUrl}/${forgejoOrg}/${e.r.name}";
    repository = {
      name = key;
      value = {
        name = e.r.name;
        description = "READ-ONLY MIRROR";
        homepage_url = codebergUrl;
        visibility = "public";
        has_issues = false;
        has_projects = false;
        has_wiki = false;
        has_discussions = false;
      };
    };
    deployKey = {
      name = "${key}_mirror_deploy_key";
      value = {
        repository = "\${github_repository.${key}.name}";
        title = "Codeberg Mirroring";
        key = "\${restapi_object.${key}_push_mirror.api_data.public_key}";
        read_only = false;
      };
    };
    ruleset = {
      name = "${key}_ruleset";
      value = {
        name = "Default";
        repository = "\${github_repository.${key}.name}";
        target = "branch";
        enforcement = "active";
        conditions.ref_name = {
          include = [ "~ALL" ];
          exclude = [ ];
        };
        bypass_actors = [
          {
            actor_id = "\${tonumber(element(split(\":\", github_repository_deploy_key.${key}_mirror_deploy_key.id), 1))}";
            actor_type = "DeployKey";
            bypass_mode = "always";
          }
        ];
        rules = {
          creation = true;
          update = true;
          deletion = true;
          non_fast_forward = true;
          pull_request.required_approving_review_count = 1;
        };
      };
    };
    actions = {
      name = "${key}_actions";
      value = {
        repository = "\${github_repository.${key}.name}";
        enabled = false;
      };
    };
  }) allRepos;

  teamEntries = map (t: {
    name = t.slug;
    value = {
      inherit (t) name;
      privacy = "closed";
    };
  }) teams;

  membershipEntries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
      teamId = "\${github_team.${slug}.id}";
      all = (gov.groupMembers t) ++ builtins.concatMap (p: gov.groupMembers p) (t.projects or [ ]);
    in
    map (
      username:
      let
        key = underscore username;
      in
      {
        name = "${slug}_${key}";
        value = {
          team_id = teamId;
          username = "\${data.external.identity_${key}.result.github}";
          role = if builtins.elem username (t.leads or [ ]) then "maintainer" else "member";
        };
      }
    ) all
  ) teams;
in
lib.optionalAttrs hasGithub {
  github_repos = {
    resource = {
      github_repository = builtins.listToAttrs (map (r: r.repository) repoResources);
      github_repository_deploy_key = builtins.listToAttrs (map (r: r.deployKey) repoResources);
      github_repository_ruleset = builtins.listToAttrs (map (r: r.ruleset) repoResources);
      github_actions_repository_permissions = builtins.listToAttrs (map (r: r.actions) repoResources);
    };
  };
  github_teams = {
    resource.github_team = builtins.listToAttrs teamEntries;
  };
  github_memberships = {
    resource.github_team_membership = builtins.listToAttrs membershipEntries;
  };
}
