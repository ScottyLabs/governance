{ gov }:
let
  gg = gov.org.google_groups;
  td = gov.org.tech_director;

  userKey = u: builtins.replaceStrings [ "-" ] [ "_" ] u;

  devopsLeads = builtins.concatMap (t: t.leads or [ ]) (
    builtins.filter (t: t.slug == "devops") gov.teams
  );

  withTd = leads: [ td ] ++ (builtins.filter (l: l != td) leads);

  adminMembers = withTd devopsLeads;
  opsMembers = withTd gov.allLeads;
  techMembers = gov.allMembers;

  lookup = key: email: {
    "${key}" = {
      group_key.id = email;
    };
  };

  memberships =
    key: members:
    builtins.listToAttrs (
      map (u: {
        name = "${key}_${userKey u}";
        value = {
          group = "\${data.google_cloud_identity_group_lookup.${key}.name}";
          preferred_member_key.id = "\${data.external.identity_${userKey u}.result.cmu-saml}@andrew.cmu.edu";
          roles =
            if u == td then
              [
                { name = "MEMBER"; }
                { name = "MANAGER"; }
              ]
            else
              [ { name = "MEMBER"; } ];
        };
      }) members
    );
in
{
  google_groups = {
    data.google_cloud_identity_group_lookup =
      (lookup "admin" gg.admin) // (lookup "ops" gg.ops) // (lookup "tech" gg.tech);
    resource.google_cloud_identity_group_membership =
      (memberships "admin" adminMembers)
      // (memberships "ops" opsMembers)
      // (memberships "tech" techMembers);
  };
}
