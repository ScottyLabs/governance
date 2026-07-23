{ lib, gov }:
let
  inherit (gov)
    teams
    hasOwnGroup
    groupMembers
    allMembers
    ;

  realmId = "\${data.keycloak_realm.this.id}";
  projectsId = "\${keycloak_group.projects.id}";

  userId =
    u: "\${data.external.identity_${builtins.replaceStrings [ "-" ] [ "_" ] u}.result.cmu-saml}";

  groupEntries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
      own = lib.optionals (hasOwnGroup t) [
        {
          name = "project_${slug}";
          value = {
            realm_id = realmId;
            parent_id = projectsId;
            name = slug;
          };
        }
        {
          name = "project_${slug}_admins";
          value = {
            realm_id = realmId;
            parent_id = "\${keycloak_group.project_${slug}.id}";
            name = "admins";
          };
        }
      ];
      projs = builtins.concatMap (
        p:
        let
          gk = "project_${slug}_${p.slug}";
        in
        [
          {
            name = gk;
            value = {
              realm_id = realmId;
              parent_id = projectsId;
              name = p.slug;
            };
          }
          {
            name = "${gk}_admins";
            value = {
              realm_id = realmId;
              parent_id = "\${keycloak_group.${gk}.id}";
              name = "admins";
            };
          }
        ]
      ) (t.projects or [ ]);
    in
    own ++ projs
  ) teams;

  emit = gk: members: leads: [
    {
      name = gk;
      value = {
        realm_id = realmId;
        group_id = "\${keycloak_group.${gk}.id}";
        members = map userId members;
      };
    }
    {
      name = "${gk}_admins";
      value = {
        realm_id = realmId;
        group_id = "\${keycloak_group.${gk}_admins.id}";
        members = map userId leads;
      };
    }
  ];

  membershipEntries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
      teamAll = groupMembers t;
      teamLeads = t.leads or [ ];
      own = lib.optionals (hasOwnGroup t) (emit "project_${slug}" teamAll teamLeads);
      projs = builtins.concatMap (
        p: emit "project_${slug}_${p.slug}" (teamAll ++ (groupMembers p)) (teamLeads ++ (p.leads or [ ]))
      ) (t.projects or [ ]);
    in
    own ++ projs
  ) teams;

  projectsMembership = {
    name = "projects";
    value = {
      realm_id = realmId;
      group_id = projectsId;
      members = map userId allMembers;
    };
  };

  kc = gov.org.keycloak or null;
  dashToUs = builtins.replaceStrings [ "-" ] [ "_" ];
  hasOidc = r: (r.features or { }) ? oidc_client;
  isAdmin = r: (r.features.oidc_client.admin or false);

  mkClient =
    {
      key,
      name,
      redirect,
    }:
    [
      {
        rtype = "keycloak_openid_client";
        k = key;
        body = {
          realm_id = realmId;
          client_id = name;
          inherit name;
          access_type = "CONFIDENTIAL";
          standard_flow_enabled = true;
          direct_access_grants_enabled = false;
          valid_redirect_uris = [ redirect ];
          web_origins = [ "+" ];
        };
      }
      {
        rtype = "keycloak_openid_group_membership_protocol_mapper";
        k = key;
        body = {
          realm_id = realmId;
          client_id = "\${keycloak_openid_client.${key}.id}";
          name = "groups";
          claim_name = "groups";
          full_path = true;
          add_to_id_token = true;
          add_to_access_token = true;
          add_to_userinfo = true;
        };
      }
    ];

  mkAdminClient =
    { key, name }:
    [
      {
        rtype = "keycloak_openid_client";
        k = key;
        body = {
          realm_id = realmId;
          client_id = name;
          inherit name;
          access_type = "CONFIDENTIAL";
          standard_flow_enabled = false;
          direct_access_grants_enabled = false;
          service_accounts_enabled = true;
          valid_redirect_uris = [ ];
          web_origins = [ ];
        };
      }
    ]
    ++
      map
        (role: {
          rtype = "keycloak_openid_client_service_account_role";
          k = "${key}_${dashToUs role}";
          body = {
            realm_id = realmId;
            service_account_user_id = "\${keycloak_openid_client.${key}.service_account_user_id}";
            client_id = "\${data.keycloak_openid_client.realm_management.id}";
            inherit role;
          };
        })
        [
          "view-users"
          "manage-users"
          "view-identity-providers"
        ];

  mkSecret =
    {
      k,
      name,
      value,
    }:
    {
      rtype = "vault_kv_secret_v2";
      inherit k;
      body = {
        mount = "secret";
        inherit name;
        data_json = "\${jsonencode({ value = ${value} })}";
      };
    };

  mkVarSecrets =
    {
      repo,
      key,
      profile,
    }:
    map (
      s:
      mkSecret {
        k = "${key}_${profile}_${lib.toLower s.var}";
        name = "secretspec/${repo}/${profile}/${s.var}";
        inherit (s) value;
      }
    );

  oidcSecrets =
    {
      repo,
      key,
      profile,
      clientKey,
      relay,
    }:
    mkVarSecrets { inherit repo key profile; } [
      {
        var = "OIDC_CLIENT_ID";
        value = "keycloak_openid_client.${clientKey}.client_id";
      }
      {
        var = "OIDC_CLIENT_SECRET";
        value = "keycloak_openid_client.${clientKey}.client_secret";
      }
      {
        var = "KEYCLOAK_URL";
        value = builtins.toJSON kc.url;
      }
      {
        var = "KEYCLOAK_REALM";
        value = builtins.toJSON kc.realm;
      }
      {
        var = "OAUTH_RELAY_URL";
        value = builtins.toJSON relay;
      }
    ];

  groupSecrets =
    {
      repo,
      key,
      profile,
      groupPath,
    }:
    mkVarSecrets { inherit repo key profile; } [
      {
        var = "PROJECT_GROUP";
        value = builtins.toJSON groupPath;
      }
      {
        var = "PROJECT_ADMIN_GROUP";
        value = builtins.toJSON "${groupPath}/admins";
      }
    ];

  adminSecrets =
    {
      repo,
      key,
      profile,
      adminKey,
    }:
    [
      (mkSecret {
        k = "${key}_${profile}_admin_client_id";
        name = "secretspec/${repo}/${profile}/KEYCLOAK_ADMIN_CLIENT_ID";
        value = "keycloak_openid_client.${adminKey}.client_id";
      })
      (mkSecret {
        k = "${key}_${profile}_admin_client_secret";
        name = "secretspec/${repo}/${profile}/KEYCLOAK_ADMIN_CLIENT_SECRET";
        value = "keycloak_openid_client.${adminKey}.client_secret";
      })
    ];

  repoEntries =
    groupPath: repo:
    let
      inherit (repo) name;
      key = dashToUs name;
      stagingKey = "${key}_staging";
      devKey = "${key}_dev";
      stagingId = "${name}-staging";
      devId = "${name}-dev";
      profiles = [
        {
          profile = "prod";
          clientKey = key;
          relay = kc.redirect_uri;
        }
        {
          profile = "staging";
          clientKey = stagingKey;
          relay = kc.redirect_uri;
        }
        {
          profile = "preview";
          clientKey = stagingKey;
          relay = kc.redirect_uri;
        }
        {
          profile = "dev";
          clientKey = devKey;
          relay = kc.dev_redirect_uri;
        }
      ];
      clients =
        mkClient {
          inherit key name;
          redirect = kc.redirect_uri;
        }
        ++ mkClient {
          key = stagingKey;
          name = stagingId;
          redirect = kc.redirect_uri;
        }
        ++ mkClient {
          key = devKey;
          name = devId;
          redirect = kc.dev_redirect_uri;
        };
      oidc = builtins.concatMap (
        p:
        oidcSecrets {
          inherit key;
          repo = name;
          inherit (p) profile;
          inherit (p) clientKey;
          inherit (p) relay;
        }
      ) profiles;
      grp = lib.optionals (groupPath != null) (
        builtins.concatMap (
          p:
          groupSecrets {
            inherit key groupPath;
            repo = name;
            inherit (p) profile;
          }
        ) profiles
      );
      admin = lib.optionals (isAdmin repo) (
        mkAdminClient {
          key = "${key}_admin";
          name = "${name}-admin";
        }
        ++ builtins.concatMap (
          p:
          adminSecrets {
            inherit key;
            repo = name;
            inherit (p) profile;
            adminKey = "${key}_admin";
          }
        ) profiles
      );
    in
    clients ++ oidc ++ grp ++ admin;

  groupEntriesFor =
    { group, path }:
    builtins.concatMap (repoEntries path) (builtins.filter hasOidc (group.repos or [ ]));

  clientEntries = builtins.concatMap (
    t:
    let
      teamGroup = {
        group = t;
        path = if hasOwnGroup t then "/projects/${t.slug}" else null;
      };
      projGroups = map (p: {
        group = p;
        path = "/projects/${p.slug}";
      }) (t.projects or [ ]);
    in
    builtins.concatMap groupEntriesFor ([ teamGroup ] ++ projGroups)
  ) teams;

  needsRealm = builtins.any (
    e: e.rtype == "keycloak_openid_client_service_account_role"
  ) clientEntries;

  clientResource = builtins.mapAttrs (
    _: es:
    builtins.listToAttrs (
      map (e: {
        name = e.k;
        value = e.body;
      }) es
    )
  ) (lib.groupBy (e: e.rtype) clientEntries);
in
{
  keycloak_groups = {
    resource.keycloak_group = builtins.listToAttrs groupEntries;
  };
  keycloak_memberships = {
    resource.keycloak_group_memberships = builtins.listToAttrs (
      membershipEntries ++ [ projectsMembership ]
    );
  };
  keycloak_clients = {
    resource = clientResource;
  }
  // lib.optionalAttrs needsRealm {
    data.keycloak_openid_client.realm_management = {
      realm_id = realmId;
      client_id = "realm-management";
    };
  };
}
