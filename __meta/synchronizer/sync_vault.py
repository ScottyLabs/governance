import hvac
import os
from utils import Styler


class VaultManager:
    VAULT_URL = "https://secrets.scottylabs.org"
    ADMIN_GROUP_SUFFIX = "-admins"
    DEV_GROUP_SUFFIX = "-devs"

    def __init__(self, teams):
        self.teams = teams
        self.client = hvac.Client(url=self.VAULT_URL, token=os.getenv("VAULT_TOKEN"))

        # Get the list of all groups
        data = self.client.secrets.identity.list_groups()["data"]
        self.groups_names = [data["key_info"][key]["name"] for key in data["keys"]]

        # Get the oidc mount accessor
        auth_methods = self.client.sys.list_auth_methods()
        self.oidc_mount = auth_methods.get("oidc/")["accessor"]

    def sync(self):
        with Styler("Vault"):
            for team_slug in self.teams.keys():
                self.sync_team(team_slug)

    # Sync the dev and lead groups for a team
    def sync_team(self, team_slug):
        print(f"\nSyncing team {team_slug}...")

        self.sync_group(
            team_slug,
            f"{team_slug}{self.DEV_GROUP_SUFFIX}",
            create_policy=self.create_dev_policy,
        )
        self.sync_group(
            team_slug,
            f"{team_slug}{self.ADMIN_GROUP_SUFFIX}",
            create_policy=self.create_admin_policy,
        )

    # If a group does not exist, create it and add the policy and alias to it
    def sync_group(self, team_slug, group_name, create_policy):
        if group_name not in self.groups_names:
            print(f"Creating group {group_name}...")
            policy_name = create_policy(team_slug)
            group = self.client.secrets.identity.create_or_update_group(
                name=group_name, group_type="external", policies=[policy_name]
            )
            group_id = group["data"]["id"]
            self.client.secrets.identity.create_or_update_group_alias(
                name=group_name, canonical_id=group_id, mount_accessor=self.oidc_mount
            )

    # Create the policy for the dev group
    # Devs can read and list the local secrets
    def create_dev_policy(self, team_slug):
        policy_name = f"{team_slug}{self.DEV_GROUP_SUFFIX}"
        policy_rules = f"""\
path "/ScottyLabs/data/{team_slug}/local/*" {{
    capabilities = ["read", "list"]
}}
"""
        self.client.sys.create_or_update_acl_policy(
            name=policy_name, policy=policy_rules
        )
        return policy_name

    # Create the policy for the lead group
    # Leads can read, create, update, delete, list, and sudo the secrets
    def create_admin_policy(self, team_slug):
        policy_name = f"{team_slug}{self.ADMIN_GROUP_SUFFIX}"
        policy_rules = f"""\
path "/ScottyLabs/data/{team_slug}/*" {{
    capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}}

path "/ScottyLabs/metadata/{team_slug}/*" {{
    capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}}
"""
        self.client.sys.create_or_update_acl_policy(
            name=policy_name, policy=policy_rules
        )
        return policy_name
