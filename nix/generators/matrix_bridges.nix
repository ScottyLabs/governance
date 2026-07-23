{ lib, gov }:
let
  hasComm = gov.org ? communication;
  comm = gov.org.communication or { };
  orgName = gov.org.name;

  mkSpec = teamName: teamSlug: projName: projSlug: slack: discord: {
    inherit
      teamName
      teamSlug
      projName
      projSlug
      slack
      discord
      ;
  };

  orgSpecs = lib.optionals hasComm (
    [
      (mkSpec orgName "org" "Hub" "hub" comm.slack_hub_channel_id comm.discord_hub_channel_id)
      (mkSpec orgName "org" "Tech Leads" "leads" comm.slack_leads_channel_id
        comm.discord_leads_channel_id
      )
    ]
    ++ map (c: mkSpec orgName "org" c.name c.slug (c.slack or null) (c.discord or null)) (
      comm.channels or [ ]
    )
  );

  teamSpecs = builtins.concatMap (
    t:
    (map (c: mkSpec t.name t.slug null null (c.slack or null) (c.discord or null)) (t.channels or [ ]))
    ++ builtins.concatMap (
      p:
      map (c: mkSpec t.name t.slug p.name p.slug (c.slack or null) (c.discord or null)) (
        p.channels or [ ]
      )
    ) (t.projects or [ ])
  ) gov.teams;

  valid = builtins.filter (s: s.slack != null && s.discord != null) (orgSpecs ++ teamSpecs);

  rname =
    teamSlug: projSlug:
    let
      raw = if projSlug == null then teamSlug else "${teamSlug}_${projSlug}";
    in
    builtins.replaceStrings [ "-" ] [ "_" ] raw;

  nullResources = builtins.listToAttrs (
    map (
      s:
      let
        rn = rname s.teamSlug s.projSlug;
      in
      {
        name = "slack_bridge_login_${rn}";
        value = {
          triggers.channel = s.slack;
          provisioner = [
            { local-exec.command = "governance slack-join --channel \${self.triggers.channel}"; }
            {
              local-exec = {
                when = "destroy";
                command = "governance slack-leave --channel \${self.triggers.channel}";
              };
            }
          ];
        };
      }
    ) valid
  );

  synapseLinks = builtins.listToAttrs (
    map (
      s:
      let
        rn = rname s.teamSlug s.projSlug;
      in
      {
        name = rn;
        value = {
          discord_channel_id = s.discord;
          slack_channel_id = s.slack;
          team_name = s.teamName;
          team_slug = s.teamSlug;
          depends_on = [ "null_resource.slack_bridge_login_${rn}" ];
        }
        // lib.optionalAttrs (s.projName != null) { project_name = s.projName; }
        // lib.optionalAttrs (s.projSlug != null) { project_slug = s.projSlug; };
      }
    ) valid
  );

  resourceTypes =
    (lib.optionalAttrs (nullResources != { }) { null_resource = nullResources; })
    // (lib.optionalAttrs (synapseLinks != { }) { synapse_mautrix_slack_link = synapseLinks; });
in
{
  matrix_bridges =
    (lib.optionalAttrs hasComm { locals.matrix_slack_team_id = comm.slack_team_id; })
    // (lib.optionalAttrs (resourceTypes != { }) { resource = resourceTypes; });
}
