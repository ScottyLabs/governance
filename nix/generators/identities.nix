{ gov }:
let
  inherit (gov) allMembers;

  body = username: {
    program = [
      "governance"
      "--data-dir"
      "../data"
      "resolve-identity"
    ];
    query.codeberg_user = username;
  };

  entries = builtins.listToAttrs (
    map (u: {
      name = "identity_${builtins.replaceStrings [ "-" ] [ "_" ] u}";
      value = body u;
    }) allMembers
  );
in
{
  identities = {
    data.external = entries;
  };
}
