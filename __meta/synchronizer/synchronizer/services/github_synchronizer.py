from collections.abc import Callable
from typing import Literal

from github.GithubException import UnknownObjectException
from github.NamedUser import NamedUser
from github.Team import Team as GithubTeam

from synchronizer.clients import get_github_client
from synchronizer.logger import (
    get_app_logger,
    log_operation,
    log_team_sync,
    print_section,
)
from synchronizer.models import Contributor, Team

from .abstract_synchronizer import AbstractSynchronizer


class GithubSynchronizer(AbstractSynchronizer):
    ADMIN_SUFFIX = " Admins"

    # We can have all teams visible to all members of the organization.
    TEAM_PRIVACY = "closed"  # one of "secret" | "closed"

    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        """Initialize the GithubManager with GitHub org."""
        self.logger = get_app_logger()

        self.contributors = contributors
        self.teams = teams
        self.g = get_github_client()
        self.org = self.g.get_organization("ScottyLabs")

        # get all owners of the organization
        self.org_owners = {owner.login for owner in self.org.get_members(role="admin")}

    def sync(self) -> None:
        print_section("Github")
        self.sync_contributors()
        for team in self.teams.values():
            self.sync_team(team)

    def sync_contributors(self) -> None:
        """Sync contributors to the GitHub organization."""
        # Get all existing members
        self.existing_members = {member.login for member in self.org.get_members()}
        self.logger.debug(
            "There are %d existing members.\n", len(self.existing_members)
        )

        # Get all invited contributors
        invitations = self.org.invitations()
        invited = set()
        for invite in invitations:
            invited.add(invite.login)

        # Invite new contributors to the GitHub organization
        for github_username in self.contributors:
            if (
                github_username not in self.existing_members
                and github_username not in invited
            ):
                log_message = f"add {github_username} to GitHub organization"
                with log_operation(log_message):
                    user = self.g.get_user(github_username)
                    if not isinstance(user, NamedUser):
                        msg = f"User {github_username} is not a valid GitHub user"
                        self.logger.error(msg)
                        continue

                    self.org.invite_user(user=user, role="direct_member")

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        """Sync the team to the GitHub organization."""
        if not team.sync_github:
            self.logger.debug(
                "Team %s opted out of syncing to GitHub, skipping...\n",
                team.name,
            )
            return

        # Get or create the team and the admin team
        github_team = self.get_or_create_main_team(team.name)
        admin_team_name = f"{team.name}{self.ADMIN_SUFFIX}"
        if not github_team:
            return

        github_admin_team = self.get_or_create_admin_team(github_team, admin_team_name)
        if not github_admin_team:
            return

        # Sync the team maintainers to the GitHub admin team
        maintainers = set(team.maintainers)
        self.sync_github_admin_team(
            github_admin_team, maintainers, remove_unlisted=team.remove_unlisted
        )

        # Sync the team maintainers and contributors to the GitHub main team,
        # where devs are the contributors who are not maintainers.
        contributors = set(team.contributors)

        # Special Case: maintainers of any team need to be added to the governance
        # github main team so they can merge PRs to accept new contributors.
        if team.slug == "governance":
            for other_team in self.teams.values():
                if other_team.slug != team.slug:
                    contributors.update(other_team.maintainers)

        self.sync_github_main_team(
            github_team, maintainers, contributors, remove_unlisted=team.remove_unlisted
        )

        # Sync the repositories to the Github team
        repos = set(team.repos)
        self.sync_repos(
            github_team, github_admin_team, repos, remove_unlisted=team.remove_unlisted
        )

    def get_or_create_main_team(self, team_name: str) -> GithubTeam | None:
        """Get or create the Github main team."""
        return self.get_or_create_team(
            team_name,
            lambda name: self.org.create_team(name=name, privacy=self.TEAM_PRIVACY),
        )

    def get_or_create_admin_team(
        self, github_team: GithubTeam, admin_team_name: str
    ) -> GithubTeam | None:
        """Get or create the Github admin team, which is a subteam of the main team."""
        return self.get_or_create_team(
            admin_team_name,
            lambda name: self.org.create_team(
                name=name,
                parent_team_id=github_team.id,
                privacy=self.TEAM_PRIVACY,
            ),
        )

    def get_or_create_team(
        self,
        team_name: str,
        create_team_func: Callable[[str], GithubTeam],
    ) -> GithubTeam | None:
        """Get or create the Github team."""
        team_slug = self.get_team_slug(team_name)
        try:
            return self.org.get_team_by_slug(team_slug)
        except UnknownObjectException:
            with log_operation(f"create {team_name} GitHub team"):
                return create_team_func(team_name)
        except Exception as e:
            msg = f"Error getting {team_slug} GitHub team: {e}"
            self.logger.exception(msg)
            return None

    # https://docs.github.com/en/rest/teams/teams?apiVersion=2022-11-28#get-a-team-by-name
    def get_team_slug(self, team_name: str) -> str:
        return team_name.replace(" ", "-").lower()

    def sync_github_admin_team(
        self,
        github_admin_team: GithubTeam,
        desired_members: set[str],
        *,
        remove_unlisted: bool,
    ) -> None:
        """Sync the team maintainers as maintainers to the GitHub admin team."""
        current_members = {member.login for member in github_admin_team.get_members()}

        # Calculate uninvited new members
        new_uninvited_members = self.subtract_invited_members(
            desired_members - current_members,
            github_admin_team,
        )
        self.logger.debug(
            "Found %d new uninvited maintainer for the %s team.\n",
            len(new_uninvited_members),
            github_admin_team.name,
        )

        # Add uninvited new members
        for username in new_uninvited_members:
            self.add_or_update_member_to_team(github_admin_team, username, "maintainer")

        # Remove extra members if the team want to remove unlisted members
        if remove_unlisted:
            for username in current_members - desired_members:
                self.remove_member_from_team(github_admin_team, username)

    def sync_github_main_team(
        self,
        github_team: GithubTeam,
        maintainers: set[str],
        contributors: set[str],
        *,
        remove_unlisted: bool,
    ) -> None:
        """Sync the team members to the Github main team."""
        # Sync the team maintainers as maintainers to the team
        self.sync_members_to_team(github_team, maintainers, "maintainer")

        # Sync the team contributors who are not maintainers as members to the team
        self.sync_members_to_team(github_team, maintainers - contributors, "member")

        # Remove extra members if the team want to remove unlisted members
        if not remove_unlisted:
            self.logger.debug(
                "Team %s opted out of removing unlisted members, skipping...\n",
                github_team.name,
            )
            return

        self.remove_unlisted_members_from_main_team(github_team, contributors)

    def sync_members_to_team(
        self,
        github_team: GithubTeam,
        members: set[str],
        role: Literal["member", "maintainer"],
    ) -> None:
        """Sync the members to the Github team as the given role."""
        # Calculate new uninvited members
        current_members = {member.login for member in github_team.get_members()}
        new_members = members - current_members
        new_uninvited_members = self.subtract_invited_members(new_members, github_team)
        self.logger.debug(
            "Found %d new uninvited %s for the %s team.\n",
            len(new_uninvited_members),
            role,
            github_team.name,
        )

        # Check every member who are not invited to the team since
        # we also need to check existing members for their roles.
        # This does mean that invited members might not have the correct role, but
        # it can be corrected during the sync after the invitation is accepted anyways.
        for username in self.subtract_invited_members(members, github_team):
            try:
                current_role = github_team.get_team_membership(username).role

                # Skip role sync for organization owners since their role cannot
                # be changed and will always remain as maintainer.
                if username in self.org_owners:
                    continue

                if current_role != role:
                    self.add_or_update_member_to_team(github_team, username, role)

            except UnknownObjectException:
                self.add_or_update_member_to_team(github_team, username, role)

            except Exception:
                self.logger.exception(
                    "Error syncing %s to %s GitHub team.",
                    username,
                    github_team.name,
                )

    def subtract_invited_members(
        self, members: set[str], github_team: GithubTeam
    ) -> set[str]:
        """
        Subtract the invited members from the members list.

        We avoid sending duplicate invitations, even if the member might not have the
        correct role, because the invitation role cannot be easily obtained and the
        role can be corrected during the sync after the invitation is accepted anyways.
        """
        invitations = github_team.invitations()
        invited_members = {invitation.login for invitation in invitations}
        return members - invited_members

    def remove_unlisted_members_from_main_team(
        self, github_team: GithubTeam, desired_members: set[str]
    ) -> None:
        """Remove unlisted members from the Github main team."""
        current_members = {member.login for member in github_team.get_members()}
        unlisted_members = current_members - desired_members
        self.logger.debug(
            "Found %d unlisted members for the %s team.\n",
            len(unlisted_members),
            github_team.name,
        )

        # Remove unlisted members
        for username in unlisted_members:
            self.remove_member_from_team(github_team, username)

    def add_or_update_member_to_team(
        self,
        github_team: GithubTeam,
        username: str,
        role: Literal["member", "maintainer"],
    ) -> None:
        with log_operation(
            f"add/update {username} as a {role} to {github_team.name} GitHub team",
        ):
            user = self.g.get_user(username)
            if not isinstance(user, NamedUser):
                msg = f"User {username} is not a valid GitHub user"
                self.logger.error(msg)
                return

            github_team.add_membership(user, role=role)

    def remove_member_from_team(self, github_team: GithubTeam, username: str) -> None:
        with log_operation(f"remove {username} from {github_team.name} GitHub team"):
            user = self.g.get_user(username)
            if not isinstance(user, NamedUser):
                msg = f"User {username} is not a valid GitHub user"
                self.logger.error(msg)
                return

            github_team.remove_membership(user)

    def sync_repos(
        self,
        github_team: GithubTeam,
        github_admin_team: GithubTeam,
        repos: set[str],
        *,
        remove_unlisted: bool,
    ) -> None:
        """
        Sync the repositories to the Github team.

        Give main team write access and admin team admin access to the repository.
        """
        github_repos = github_team.get_repos()
        github_repos_names = {repo.full_name for repo in github_repos}

        # Calculate new repos
        new_repos = repos - github_repos_names
        self.logger.debug(
            "Found %d new repos for the %s team.\n",
            len(new_repos),
            github_team.name,
        )

        # Give team devs write access and team maintainers admin access to the repos
        for repo in new_repos:
            log_message = f"add {repo} to {github_team.name} Github team"
            with log_operation(log_message):
                github_team.add_to_repos(repo)

            # Always update the team repository permissions, idempotent and cheap
            github_team.update_team_repository(repo, "push")
            github_admin_team.update_team_repository(repo, "admin")

        # Remove any repositories from the Github team that are not in the team list
        # if the team want to remove unlisted repos.
        if not remove_unlisted:
            self.logger.debug(
                "Team %s opted out of removing unlisted repos, skipping...\n",
                github_team.name,
            )
            return

        # Calculate unlisted repos
        unlisted_repos = github_repos_names - repos
        self.logger.debug(
            "Found %d unlisted repos for the %s team.\n",
            len(unlisted_repos),
            github_team.name,
        )

        # Remove unlisted repos
        for repo in unlisted_repos:
            log_message = f"remove {repo} from {github_team.name} Github team"
            with log_operation(log_message):
                github_team.remove_from_repos(repo)
