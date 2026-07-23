{ lib, gov }:
let
  inherit (gov) teams;

  underscore = s: builtins.replaceStrings [ "-" ] [ "_" ] s;

  reposOf = t: (t.repos or [ ]) ++ builtins.concatMap (p: p.repos or [ ]) (t.projects or [ ]);

  hasCdn = r: (r.features or { }) ? cdn;

  entries = builtins.concatMap (
    t:
    map (repo: {
      key = "${t.slug}_${underscore repo.name}";
      bucket = "cdn-${repo.name}";
      repoName = repo.name;
    }) (builtins.filter hasCdn (reposOf t))
  ) teams;

  bucketRes = builtins.listToAttrs (
    map (e: {
      name = "cdn_${e.key}";
      value = {
        global_alias = e.bucket;
        website_enabled = true;
        website_index_document = "index.html";
      };
    }) entries
  );

  keyRes = builtins.listToAttrs (
    map (e: {
      name = "cdn_${e.key}";
      value = {
        name = e.bucket;
      };
    }) entries
  );

  permRes = builtins.listToAttrs (
    map (e: {
      name = "cdn_${e.key}";
      value = {
        bucket_id = "\${garage_bucket.cdn_${e.key}.id}";
        access_key_id = "\${garage_key.cdn_${e.key}.id}";
        read = true;
        write = true;
        owner = true;
      };
    }) entries
  );

  profiles = [
    "prod"
    "staging"
    "preview"
    "dev"
  ];

  secretVars = e: [
    {
      name = "CDN_S3_ENDPOINT";
      value = "var.garage_s3_endpoint";
    }
    {
      name = "CDN_S3_BUCKET";
      value = "\"${e.bucket}\"";
    }
    {
      name = "CDN_ACCESS_KEY_ID";
      value = "garage_key.cdn_${e.key}.id";
    }
    {
      name = "CDN_SECRET_ACCESS_KEY";
      value = "garage_key.cdn_${e.key}.secret_access_key";
    }
    {
      name = "CDN_PUBLIC_URL";
      value = "\"\${var.cdn_base_url}/${e.repoName}/\"";
    }
  ];

  vaultRes = builtins.listToAttrs (
    builtins.concatMap (
      e:
      builtins.concatMap (
        v:
        map (profile: {
          name = "${e.key}_${profile}_${lib.toLower v.name}";
          value = {
            mount = "secret";
            name = "secretspec/${e.repoName}/${profile}/${v.name}";
            data_json = "\${jsonencode({ value = ${v.value} })}";
          };
        }) profiles
      ) (secretVars e)
    ) entries
  );
in
{
  cdn = {
    resource = {
      garage_bucket = bucketRes;
      garage_key = keyRes;
      garage_bucket_permission = permRes;
      vault_kv_secret_v2 = vaultRes;
    };
  };
}
