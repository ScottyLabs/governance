import os
from collections.abc import Callable

import hvac

from synchronizer.types.team import Team
from synchronizer.utils import log_operation, log_team_sync, print_section


class VaultManager:
    VAULT_URL = "https://secrets.scottylabs.org"
    ADMIN_GROUP_SUFFIX = "-admins"
    DEV_GROUP_SUFFIX = "-devs"
    APPLICANT_GROUP_SUFFIX = "-applicants"
    APPLICANTS_FOLDER_NAME = "applicants"

    def __init__(self, teams: dict[str, Team]) -> None:
        self.teams = teams
        self.client = hvac.Client(url=self.VAULT_URL, token=os.getenv("VAULT_TOKEN"))

        # Get the list of all groups
        data = self.client.secrets.identity.list_groups()["data"]
        self.groups_names = [data["key_info"][key]["name"] for key in data["keys"]]

        # Get the oidc mount accessor
        auth_methods = self.client.sys.list_auth_methods()
        self.oidc_mount = auth_methods.get("oidc/")["accessor"]

    def sync(self) -> None:
        print_section("Vault")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        team_slug = team["slug"]
        self.sync_group(
            team_slug,
            f"{team_slug}{self.ADMIN_GROUP_SUFFIX}",
            create_policy=self.create_admin_policy,
        )
        self.sync_group(
            team_slug,
            f"{team_slug}{self.DEV_GROUP_SUFFIX}",
            create_policy=self.create_dev_policy,
        )
        self.sync_group(
            team_slug,
            f"{team_slug}{self.APPLICANT_GROUP_SUFFIX}",
            create_policy=self.create_applicant_policy,
        )

    def sync_group(
        self,
        team_slug: str,
        group_name: str,
        create_policy: Callable[[str], str],
    ) -> None:
        """Create the Vault group if it does not exist."""
        if group_name not in self.groups_names:
            with log_operation(f"create Vault group {group_name}"):
                # Create the policy
                policy_name = create_policy(team_slug)

                # Create the group
                group = self.client.secrets.identity.create_or_update_group(
                    name=group_name,
                    group_type="external",
                    policies=[policy_name],
                )

                # Create the group alias
                group_id = group["data"]["id"]
                self.client.secrets.identity.create_or_update_group_alias(
                    name=group_name,
                    canonical_id=group_id,
                    mount_accessor=self.oidc_mount,
                )

    def create_admin_policy(self, team_slug: str) -> str:
        """
        Create the policy for the admin group.

        Leads can read, create, update, delete, list, and sudo the secrets.
        """
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
            name=policy_name,
            policy=policy_rules,
        )
        return policy_name

    def create_dev_policy(self, team_slug: str) -> str:
        """
        Create the policy for the dev group.

        Devs can read and list the local secrets.
        """
        policy_name = f"{team_slug}{self.DEV_GROUP_SUFFIX}"
        policy_rules = f"""\
path "/ScottyLabs/data/{team_slug}/local/*" {{
    capabilities = ["read", "list"]
}}
"""
        self.client.sys.create_or_update_acl_policy(
            name=policy_name,
            policy=policy_rules,
        )
        return policy_name

    def create_applicant_policy(self, team_slug: str) -> str:
        """
        Create the policy for the applicant group.

        Applicants can read and list the applicant secrets.
        """
        policy_name = f"{team_slug}{self.APPLICANT_GROUP_SUFFIX}"
        policy_rules = f"""\
path "/ScottyLabs/data/{team_slug}/{self.APPLICANTS_FOLDER_NAME}/*" {{
    capabilities = ["read", "list"]
}}
"""
        self.client.sys.create_or_update_acl_policy(
            name=policy_name,
            policy=policy_rules,
        )
        return policy_name
