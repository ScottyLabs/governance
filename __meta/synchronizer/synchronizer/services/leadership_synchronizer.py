from github.NamedUser import NamedUser

from synchronizer.clients import get_github_client
from synchronizer.logger import get_app_logger, log_operation, print_section
from synchronizer.models import Team

from .abstract_synchronizer import AbstractSynchronizer


class LeadershipSynchronizer(AbstractSynchronizer):
    def __init__(self, teams: dict[str, Team]) -> None:
        self.logger = get_app_logger()
        if "leadership" not in teams:
            self.logger.warning("Leadership team not found, skipping...\n")
            return

        self.team = teams["leadership"]
        self.g = get_github_client()
        self.org = self.g.get_organization("ScottyLabs")

    def sync(self) -> None:
        print_section("Leadership")
        self.sync_github()

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
