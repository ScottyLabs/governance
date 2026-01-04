from collections.abc import Callable

from github.NamedUser import NamedUser

from synchronizer.clients import get_github_client
from synchronizer.clients.vault_client import get_vault_client
from synchronizer.logger import get_app_logger, log_operation, print_section
from synchronizer.models import Team

from .abstract_synchronizer import AbstractSynchronizer


class LeadershipSynchronizer(AbstractSynchronizer):
    """Synchronize the leadership team to the GitHub organization and Vault."""

    ADMINS_POLICY_NAME = "leadership-admins"
    DEVS_POLICY_NAME = "leadership-devs"

    def __init__(self, teams: dict[str, Team]) -> None:
        self.logger = get_app_logger()
        if "leadership" not in teams:
            self.logger.warning("Leadership team not found, skipping...\n")
            return

        self.team = teams["leadership"]
        self.g = get_github_client()
        self.org = self.g.get_organization("ScottyLabs")
        self.vault_client = get_vault_client()

    def sync(self) -> None:
        print_section("Leadership")
        self.sync_github()
        self.sync_vault()

    def sync_github(self) -> None:
        """Maintainers of the leadership team are the owners of GitHub organization."""
        current_owners = {member.login for member in self.org.get_members(role="admin")}
        desired_owners = set(self.team.maintainers)

        # Calculate new members
        new_owners = desired_owners - current_owners
        self.logger.debug(
            "Found %d new GitHub organization owners.\n",
            len(new_owners),
        )

        # Add new members
        for owner in new_owners:
            with log_operation(f"add {owner} as a GitHub organization owner"):
                user = self.g.get_user(owner)
                if not isinstance(user, NamedUser):
                    msg = f"User {owner} is not a valid GitHub user"
                    self.logger.error(msg)
                    continue

                self.org.add_to_members(user, role="admin")

        # Skip removing unlisted members if the team opted out
        if not self.team.remove_unlisted:
            self.logger.debug(
                "Team %s opted out of removing unlisted GitHub members, skipping...\n",
                self.team.name,
            )
            return

        # Set unlisted members to normal members
        for owner in current_owners - desired_owners:
            with log_operation(
                f"set previously unlisted owner {owner} as a GitHub organization member"
            ):
                user = self.g.get_user(owner)
                if not isinstance(user, NamedUser):
                    msg = f"User {owner} is not a valid GitHub user"
                    self.logger.error(msg)
                    continue

                self.org.add_to_members(user, role="member")

    def sync_vault(self) -> None:
        """
        Sync leadership team permissions in the Vault.

        The leadership team maintainers have all permissions in the Vault.
        The leadership team contributors who are not maintainers have read-only
        permissions to all secrets.
        """
        self.sync_policy(self.ADMINS_POLICY_NAME, self.generate_admins_policy)
        self.sync_policy(self.DEVS_POLICY_NAME, self.generate_devs_policy)

    def sync_policy(self, policy_name: str, generate_policy: Callable[[], str]) -> None:
        """Sync the policy in Vault."""
        # Get the current and desired policies
        current_policy = self.vault_client.sys.read_policy(name=policy_name)
        desired_policy = generate_policy()

        # Skip if the policy is already up to date
        if desired_policy == current_policy["rules"]:
            self.logger.debug(
                "Leadership %s policy is already up to date, skipping...\n",
                policy_name,
            )
            return

        # Update the policy
        with log_operation(f"update the leadership {policy_name} policy in Vault"):
            self.vault_client.sys.create_or_update_acl_policy(
                name=policy_name,
                policy=desired_policy,
            )

    def generate_admins_policy(self) -> str:
        """Given the leadership team maintainers all permissions in the Vault."""
        return """\
path "*" {
    capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}

path "/ScottyLabs/metadata/*" {
    capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}
"""

    def generate_devs_policy(self) -> str:
        """Give the leadership team devs read-only permissions to everything."""
        return """\
path "*" {
    capabilities = ["read", "list"]
}

path "/ScottyLabs/metadata/*" {
    capabilities = ["read", "list"]
}
"""
