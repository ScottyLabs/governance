{ lib, gov }:
let
  inherit (gov)
    teams
    groupMembers
    allMembers
    allLeads
    ;

  comm = gov.org.communication or { };
  guild = comm.discord_guild_id or "";
  hub = comm.discord_hub_channel_id or "";

  tfKey = builtins.replaceStrings [ "-" ] [ "_" ];

  techId = "\${discord_role.tech.id}";
  leadId = "\${discord_role.tech_lead.id}";
  roleRef = slug: "\${discord_role.${slug}.id}";

  channelIds =
    t:
    let
      chans = (t.channels or [ ]) ++ builtins.concatMap (p: p.channels or [ ]) (t.projects or [ ]);
    in
    map (c: c.discord) (builtins.filter (c: (c.discord or null) != null) chans);

  roleResources = {
    "tech" = {
      server_id = guild;
      name = "Tech";
    };
  }
  // builtins.listToAttrs (
    map (t: {
      name = t.slug;
      value = {
        server_id = guild;
        inherit (t) name;
      };
    }) teams
  )
  // {
    "tech_lead" = {
      server_id = guild;
      name = "Tech Lead";
    };
  };

  permResources =
    builtins.foldl'
      (
        acc: t:
        acc
        // builtins.listToAttrs (
          lib.imap0 (i: cid: {
            name = "${t.slug}_ch${toString i}";
            value = {
              channel_id = cid;
              type = "role";
              overwrite_id = roleRef t.slug;
              allow = 1024;
            };
          }) (channelIds t)
        )
      )
      {
        "tech_hub" = {
          channel_id = hub;
          type = "role";
          overwrite_id = techId;
          allow = 1024;
        };
      }
      teams;

  pushes =
    map (u: {
      key = tfKey u;
      id = techId;
    }) allMembers
    ++ builtins.concatMap (
      t:
      let
        allT = groupMembers t ++ builtins.concatMap groupMembers (t.projects or [ ]);
      in
      map (u: {
        key = tfKey u;
        id = roleRef t.slug;
      }) allT
    ) teams
    ++ map (u: {
      key = tfKey u;
      id = leadId;
    }) allLeads;

  orderedKeys = lib.lists.unique (map (p: p.key) pushes);

  memberRoles = builtins.listToAttrs (
    map (k: {
      name = k;
      value = {
        server_id = guild;
        user_id = "\${data.external.identity_${k}.result.discord_id}";
        role = map (p: { role_id = p.id; }) (builtins.filter (p: p.key == k) pushes);
      };
    }) orderedKeys
  );
in
{
  discord =
    if comm == { } || guild == "" then
      { }
    else
      {
        resource = {
          discord_role = roleResources;
          discord_channel_permission = permResources;
          discord_member_roles = memberRoles;
        };
      };
}
