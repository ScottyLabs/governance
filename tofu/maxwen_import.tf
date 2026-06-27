# Temporary maxwen ops membership import with tech presence check
# Delete after the apply lands

data "google_cloud_identity_group_memberships" "ops_drift" {
  group = data.google_cloud_identity_group_lookup.ops.name
}

data "google_cloud_identity_group_memberships" "tech_drift" {
  group = data.google_cloud_identity_group_lookup.tech.name
}

import {
  to = google_cloud_identity_group_membership.ops_tentype
  id = one([
    for m in data.google_cloud_identity_group_memberships.ops_drift.memberships : m.name
    if try(m.preferred_member_key[0].id, m.preferred_member_key.id, "") == "maxwen@andrew.cmu.edu"
  ])
}

output "tech_has_maxwen" {
  value = one([
    for m in data.google_cloud_identity_group_memberships.tech_drift.memberships : m.name
    if try(m.preferred_member_key[0].id, m.preferred_member_key.id, "") == "maxwen@andrew.cmu.edu"
  ])
}
