{ gov }:
let
  inherit (gov) org teams allLeads;

  key = u: builtins.replaceStrings [ "-" ] [ "_" ] u;

  vw = org.vaultwarden;
  orgId = vw.org_id;
  td = org.tech_director;

  devopsLeads = builtins.concatMap (t: t.leads or [ ]) (
    builtins.filter (t: t.slug == "devops") teams
  );

  allVwUsers =
    let
      base = [ td ] ++ (builtins.filter (l: l != td) devopsLeads);
      rest = builtins.filter (l: !(builtins.elem l base)) allLeads;
    in
    base ++ rest;

  memberEntries = builtins.listToAttrs (
    map (u: {
      name = "vw_${key u}";
      value = {
        organization_id = orgId;
        email = "\${data.external.identity_${key u}.result.cmu-saml}@andrew.cmu.edu";
      };
    }) allVwUsers
  );

  techMembers = [
    {
      id = "\${data.bitwarden_org_member.vw_${key td}.id}";
      manage = true;
    }
  ]
  ++ map (l: {
    id = "\${data.bitwarden_org_member.vw_${key l}.id}";
    manage = true;
  }) (builtins.filter (l: l != td) devopsLeads);

  leadMembers = map (u: {
    id = "\${data.bitwarden_org_member.vw_${key u}.id}";
    manage = true;
  }) allLeads;
in
{
  vaultwarden = {
    resource.bitwarden_org_collection = {
      tech = {
        organization_id = orgId;
        name = "Tech";
        member = techMembers;
      };
      tech_leads = {
        organization_id = orgId;
        name = "Tech/Tech Leads";
        member = leadMembers;
      };
    };
    data.bitwarden_org_member = memberEntries;
  };
}
