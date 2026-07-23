{ gov }:
let
  inherit (gov)
    teams
    groupsOf
    groupMembers
    org
    ;

  u = builtins.replaceStrings [ "-" ] [ "_" ];

  hasForgejo = org ? forgejo;
  hasGithub = org ? github;
  forgejoOrg = if hasForgejo then org.forgejo.org else "";

  defaults = org.defaults or { };
  defVisibility = defaults.repo_visibility or "public";
  defBranch = defaults.default_branch or "main";

  toAttrs = builtins.listToAttrs;

  teamRepos = t: builtins.concatMap (g: g.repos or [ ]) (groupsOf t);

  scopesOf =
    t:
    let
      teamLeads = t.leads or [ ];
      mkLeads = group: extra: extra ++ builtins.filter (l: !(builtins.elem l extra)) (group.leads or [ ]);
      first = {
        key = "${t.slug}_leads";
        name = "${t.slug}-leads";
        description = "${t.name} leads";
        team_slug = t.slug;
        group = t;
        leads = mkLeads t [ ];
      };
      projs = map (p: {
        key = "${p.slug}_leads";
        name = "${p.slug}-leads";
        description = "${p.name} leads";
        team_slug = t.slug;
        group = p;
        leads = mkLeads p teamLeads;
      }) (t.projects or [ ]);
    in
    builtins.filter (s: s.leads != [ ] && (s.group.repos or [ ]) != [ ]) ([ first ] ++ projs);

  allScopes = builtins.concatMap scopesOf teams;

  repoEntries = builtins.concatMap (
    t:
    builtins.concatMap (
      group:
      map (repo: {
        name = "${t.slug}_${u repo.name}";
        value = {
          inherit (repo) name;
          description = repo.description or (group.description or "");
          website = repo.url or (group.public_url or "");
          owner = forgejoOrg;
          auto_init = true;
          default_branch = defBranch;
          private = (repo.visibility or defVisibility) == "private";
          lifecycle = {
            ignore_changes = [ "clone_addr" ];
          };
        };
      }) (group.repos or [ ])
    ) (groupsOf t)
  ) teams;

  teamTeamEntries = map (t: {
    name = t.slug;
    value = {
      name = t.slug;
      description = t.name;
      organization_id = "\${data.forgejo_organization.this.id}";
      permission = "write";
      units_map = {
        "repo.code" = "write";
        "repo.issues" = "write";
        "repo.pulls" = "write";
      };
    };
  }) teams;

  leadTeamEntries = map (s: {
    name = s.key;
    value = {
      inherit (s) name description;
      organization_id = "\${data.forgejo_organization.this.id}";
      includes_all_repositories = false;
      can_create_org_repo = false;
      permission = "admin";
      units_map = {
        "repo.code" = "admin";
        "repo.issues" = "admin";
        "repo.pulls" = "admin";
        "repo.releases" = "admin";
        "repo.wiki" = "admin";
        "repo.ext_wiki" = "admin";
        "repo.ext_issues" = "admin";
        "repo.projects" = "admin";
        "repo.packages" = "admin";
        "repo.actions" = "admin";
      };
    };
  }) allScopes;

  memberTeamEntries = builtins.concatMap (
    t:
    let
      all = (groupMembers t) ++ (builtins.concatMap groupMembers (t.projects or [ ]));
    in
    map (username: {
      name = "${t.slug}_${u username}";
      value = {
        team_id = "\${forgejo_team.${t.slug}.id}";
        user = "\${data.external.identity_${u username}.result.codeberg}";
      };
    }) all
  ) teams;

  leadMemberEntries = builtins.concatMap (
    s:
    map (username: {
      name = "${s.key}_${u username}";
      value = {
        team_id = "\${forgejo_team.${s.key}.id}";
        user = "\${data.external.identity_${u username}.result.codeberg}";
      };
    }) s.leads
  ) allScopes;

  pushMirror = builtins.concatMap (
    t:
    map (repo: rec {
      key = "${t.slug}_${u repo.name}";
      localName = "${key}_mirror_data";
      local = {
        name = localName;
        value = {
          remote_address = "git@github.com:${org.github.org}/${repo.name}.git";
          interval = "8h0m0s";
          sync_on_commit = true;
          use_ssh = true;
        };
      };
      resource = {
        name = "${key}_push_mirror";
        value = {
          path = "/api/v1/repos/${forgejoOrg}/${repo.name}/push_mirrors";
          data = "\${jsonencode(local.${localName})}";
          id_attribute = "remote_name";
          depends_on = [ "forgejo_repository.${key}" ];
        };
      };
    }) (teamRepos t)
  ) teams;

  kennelWebhook = builtins.concatMap (
    t:
    map (repo: rec {
      key = "${t.slug}_${u repo.name}";
      localName = "${key}_kennel_webhook_data";
      local = {
        name = localName;
        value = {
          type = "forgejo";
          active = true;
          config = {
            url = "\${var.kennel_webhook_url}";
            content_type = "json";
            secret = "\${random_password.kennel_webhook_secret.result}";
          };
          events = [
            "push"
            "pull_request"
            "delete"
          ];
        };
      };
      resource = {
        name = "${key}_kennel_webhook";
        value = {
          path = "/api/v1/repos/${forgejoOrg}/${repo.name}/hooks";
          data = "\${jsonencode(local.${localName})}";
          id_attribute = "id";
          update_method = "PATCH";
          depends_on = [ "forgejo_repository.${key}" ];
        };
      };
    }) (builtins.filter (repo: (repo.features or { }) ? kennel) (teamRepos t))
  ) teams;

  docsWebhook = builtins.concatMap (
    t:
    map
      (repo: rec {
        key = "${t.slug}_${u repo.name}";
        localName = "${key}_docs_webhook_data";
        local = {
          name = localName;
          value = {
            type = "forgejo";
            active = true;
            config = {
              url = "\${var.docs_webhook_url}";
              content_type = "json";
            };
            events = [ "push" ];
          };
        };
        resource = {
          name = "${key}_docs_webhook";
          value = {
            path = "/api/v1/repos/${forgejoOrg}/${repo.name}/hooks";
            data = "\${jsonencode(local.${localName})}";
            id_attribute = "id";
            update_method = "PATCH";
            depends_on = [ "forgejo_repository.${key}" ];
          };
        };
      })
      (
        builtins.filter (repo: ((repo.features or { }) ? docs) && repo.name != "documentation") (
          teamRepos t
        )
      )
  ) teams;

  teamRepoEntries = builtins.concatMap (
    t:
    map (repo: rec {
      key = "${t.slug}_${u repo.name}";
      name = key;
      value = {
        team_id = "\${forgejo_team.${t.slug}.id}";
        owner = forgejoOrg;
        repository = repo.name;
        depends_on = [
          "forgejo_repository.${key}"
          "forgejo_team.${t.slug}"
        ];
      };
    }) (teamRepos t)
  ) teams;

  leadRepoEntries = builtins.concatMap (
    s:
    map (repo: {
      name = "${s.key}_${u repo.name}";
      value = {
        team_id = "\${forgejo_team.${s.key}.id}";
        owner = forgejoOrg;
        repository = repo.name;
        depends_on = [
          "forgejo_repository.${s.team_slug}_${u repo.name}"
          "forgejo_team.${s.key}"
        ];
      };
    }) (s.group.repos or [ ])
  ) allScopes;
in
{
  forgejo_repos = if hasForgejo then { resource.forgejo_repository = toAttrs repoEntries; } else { };

  forgejo_teams =
    if hasForgejo then
      { resource.forgejo_team = toAttrs (teamTeamEntries ++ leadTeamEntries); }
    else
      { };

  forgejo_memberships =
    if hasForgejo then
      { resource.forgejo_team_member = toAttrs (memberTeamEntries ++ leadMemberEntries); }
    else
      { };

  forgejo_push_mirrors =
    if hasForgejo && hasGithub then
      {
        locals = toAttrs (map (e: e.local) pushMirror);
        resource.restapi_object = toAttrs (map (e: e.resource) pushMirror);
      }
    else
      { };

  forgejo_kennel_webhooks =
    if hasForgejo then
      {
        locals = toAttrs (map (e: e.local) kennelWebhook);
        resource.restapi_object = toAttrs (map (e: e.resource) kennelWebhook);
      }
    else
      { };

  forgejo_docs_webhooks =
    if hasForgejo then
      {
        locals = toAttrs (map (e: e.local) docsWebhook);
        resource.restapi_object = toAttrs (map (e: e.resource) docsWebhook);
      }
    else
      { };

  forgejo_team_repos =
    if hasForgejo then
      { resource.forgejo_team_repository = toAttrs (teamRepoEntries ++ leadRepoEntries); }
    else
      { };
}
