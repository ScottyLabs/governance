{ lib, dataDir }:
let
  norm = lib.toLower;
  normList = map norm;

  normGroup =
    g:
    g
    // {
      leads = normList (g.leads or [ ]);
      members = normList (g.members or [ ]);
    };

  teamFiles = builtins.filter (lib.hasSuffix ".toml") (
    builtins.attrNames (builtins.readDir "${dataDir}/teams")
  );
  rawTeams = map (f: (fromTOML (builtins.readFile "${dataDir}/teams/${f}")).team) teamFiles;
  teams = map (t: (normGroup t) // { projects = map normGroup (t.projects or [ ]); }) rawTeams;

  inherit (fromTOML (builtins.readFile "${dataDir}/org.toml")) org;

  groupsOf = t: [ t ] ++ (t.projects or [ ]);
  groupMembers = g: (g.leads or [ ]) ++ (g.members or [ ]);

  hasOwnGroup = t: ((t.projects or [ ]) == [ ]) || ((t.repos or [ ]) != [ ]);

  uniqSorted =
    xs:
    builtins.attrNames (
      builtins.listToAttrs (
        map (x: {
          name = x;
          value = null;
        }) xs
      )
    );
  allGroups = builtins.concatMap groupsOf teams;
  allMembers = uniqSorted (builtins.concatMap groupMembers allGroups);
  allLeads = uniqSorted (builtins.concatMap (g: g.leads or [ ]) allGroups);
in
{
  inherit
    org
    teams
    norm
    groupsOf
    groupMembers
    hasOwnGroup
    allMembers
    allLeads
    ;
}
