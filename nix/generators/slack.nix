{ lib, gov }:
let
  inherit (gov)
    teams
    groupMembers
    allMembers
    allLeads
    ;

  comm = gov.org.communication or null;
  enabled = comm != null && (comm.slack_workspace or "") != "";

  key = u: builtins.replaceStrings [ "-" ] [ "_" ] u;
  slackId = u: "\${data.external.identity_${key u}.result.slack_id}";

  channelInvite = channel: user: {
    triggers = {
      inherit channel user;
    };
    provisioner = [
      {
        "local-exec".command =
          "governance slack-invite --channel \${self.triggers.channel} --user \${self.triggers.user}";
      }
      {
        "local-exec" = {
          when = "destroy";
          command = "governance slack-kick --channel \${self.triggers.channel} --user \${self.triggers.user}";
        };
      }
    ];
  };

  hubEntries = map (u: {
    name = "slack_hub_${key u}";
    value = channelInvite comm.slack_hub_channel_id (slackId u);
  }) allMembers;

  leadEntries = map (u: {
    name = "slack_leads_${key u}";
    value = channelInvite comm.slack_leads_channel_id (slackId u);
  }) allLeads;

  slackChannelIds =
    t:
    builtins.filter (x: x != null) (
      map (c: c.slack or null) (
        (t.channels or [ ]) ++ builtins.concatMap (p: p.channels or [ ]) (t.projects or [ ])
      )
    );

  teamEntries = builtins.concatMap (
    t:
    let
      inherit (t) slug;
      channelIds = slackChannelIds t;
      all = groupMembers t ++ builtins.concatMap groupMembers (t.projects or [ ]);
    in
    lib.concatLists (
      lib.imap0 (
        chIdx: channelId:
        map (u: {
          name = "slack_${slug}_ch${toString chIdx}_${key u}";
          value = channelInvite channelId (slackId u);
        }) all
      ) channelIds
    )
  ) teams;

  entries = hubEntries ++ leadEntries ++ teamEntries;
in
{
  slack = lib.optionalAttrs enabled {
    resource.null_resource = builtins.listToAttrs entries;
  };
}
