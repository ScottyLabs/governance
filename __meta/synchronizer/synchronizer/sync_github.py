import os
import traceback
from typing import Literal

from github import Auth, Github
from github.GithubException import UnknownObjectException
from github.NamedUser import NamedUser
from github.Team import Team
from utils import debug, error, log_operation, log_team_sync, print_section


class GithubManager:
    ADMIN_SUFFIX = " Admins"

    # We can have all teams visible to all members of the organization.
    TEAM_PRIVACY = "closed"  # one of "secret" | "closed"

    def __init__(self, contributors, teams):
        """Initialize the GithubManager with GitHub org."""
        github_token = os.getenv("SYNC_GITHUB_TOKEN")
        if not github_token:
            raise ValueError("SYNC_GITHUB_TOKEN is not set")

        self.contributors = contributors
        self.teams = teams
        self.g = Github(auth=Auth.Token(github_token))
        self.org = self.g.get_organization("ScottyLabs")

    def sync(self):
        print_section("Github")
        self.sync_contributors()
        for team in self.teams.values():
            self.sync_team(team)

    def sync_contributors(self):
        """Sync contributors to the GitHub organization."""
        # Get all existing members
        self.existing_members = set(member.login for member in self.org.get_members())
        debug(f"There are {len(self.existing_members)} existing members.\n")

        # Get all invited contributors
        invitations = self.org.invitations()
        invited = set()
        for invite in invitations:
            invited.add(invite.login)

        # Invite new contributors to the GitHub organization
        for github_username, _ in self.contributors.items():
            if (
                github_username not in self.existing_members
                and github_username not in invited
            ):
                log_message = f"add {github_username} to GitHub organization"
                with log_operation(log_message):
                    user = self.g.get_user(github_username)
                    if not isinstance(user, NamedUser):
                        error(f"User {github_username} is not a valid GitHub user")
                        continue
                    self.org.invite_user(user=user, role="direct_member")

    @log_team_sync()
    def sync_team(self, team):
        """Sync the team to the GitHub organization."""
        team_name = team["name"]
        remove_unlisted = team.get("remove-unlisted", True)

        # Get or create the team and the admin team
        github_team = self.get_or_create_main_team(team_name)
        admin_team_name = f"{team_name}{self.ADMIN_SUFFIX}"
        github_admin_team = self.get_or_create_admin_team(github_team, admin_team_name)

        # Sync the team leads to the GitHub admin team
        leads = set(team["leads"])
        self.sync_github_admin_team(github_admin_team, leads, remove_unlisted)

        # Sync the team leads and devs to the GitHub main team
        devs = set(team["devs"])
        self.sync_github_main_team(github_team, leads, devs, remove_unlisted)

        # Sync the repositories to the Github team
        repos = set(team["repos"])
        self.sync_repos(github_team, github_admin_team, repos, remove_unlisted)

    def get_or_create_main_team(self, team_name):
        """Get or create the Github main team."""
        return self.get_or_create_team(
            team_name,
            lambda name: self.org.create_team(name=name, privacy=self.TEAM_PRIVACY),
        )

    def get_or_create_admin_team(self, github_team, admin_team_name):
        """Get or create the Github admin team, which is a subteam of the main team."""
        return self.get_or_create_team(
            admin_team_name,
            lambda name: self.org.create_team(
                name=name, parent_team_id=github_team.id, privacy=self.TEAM_PRIVACY
            ),
        )

    def get_or_create_team(self, team_name, create_team_func):
        """Get or create the Github team."""
        team_slug = self.get_team_slug(team_name)
        try:
            return self.org.get_team_by_slug(team_slug)
        except UnknownObjectException:
            with log_operation(f"create {team_name} GitHub team"):
                return create_team_func(team_name)
        except Exception as e:
            error(f"Error getting {team_slug} GitHub team: {e}")
            traceback.print_exc()

    # https://docs.github.com/en/rest/teams/teams?apiVersion=2022-11-28#get-a-team-by-name
    def get_team_slug(self, team_name):
        return team_name.replace(" ", "-").lower()

    def sync_github_admin_team(
        self, github_admin_team, desired_members, remove_unlisted
    ):
        """Sync the team leads as maintainers to the GitHub admin team."""
        current_members = {member.login for member in github_admin_team.get_members()}

        # Calculate new members
        new_members = desired_members - current_members
        debug(
            f"Found {len(new_members)} new maintainers for the {github_admin_team.name} team."
        )

        # Calculate uninvited new members
        new_uninvited_members = self.subtract_invited_members(
            new_members, github_admin_team
        )
        debug(
            f"Found {len(new_uninvited_members)} new uninvited maintainers for the {github_admin_team.name} team."
        )

        # Add uninvited new members
        for username in new_uninvited_members:
            self.add_or_update_member_to_team(github_admin_team, username, "maintainer")

        # Remove extra members if the team want to remove unlisted members
        if remove_unlisted:
            for username in current_members - desired_members:
                self.remove_member_from_team(github_admin_team, username)

    def sync_github_main_team(self, github_team, leads, devs, remove_unlisted):
        """Sync the team members to the Github main team."""
        self.sync_members_to_team(github_team, leads, "maintainer")
        self.sync_members_to_team(github_team, devs, "member")

        # Remove extra members if the team want to remove unlisted members
        if remove_unlisted:
            desired_members = leads.union(devs)
            self.remove_unlisted_members_from_main_team(github_team, desired_members)

    def sync_members_to_team(
        self, github_team, members, role: Literal["member", "maintainer"]
    ):
        """Sync the members to the Github team as the given role."""
        # Calculate new members
        current_members = {member.login for member in github_team.get_members()}
        new_members = members - current_members
        debug(f"Found {len(new_members)} new {role}s for the {github_team.name} team.")

        # Calculate uninvited new members
        new_uninvited_members = self.subtract_invited_members(new_members, github_team)
        debug(
            f"Found {len(new_uninvited_members)} new uninvited {role}s for the {github_team.name} team."
        )

        # Add new uninvited members
        for username in self.subtract_invited_members(members, github_team):
            try:
                current_role = github_team.get_team_membership(username).role
                if current_role != role:
                    self.add_or_update_member_to_team(github_team, username, role)

            except UnknownObjectException:
                self.add_or_update_member_to_team(github_team, username, role)

            except Exception as e:
                error(
                    f"Error syncing {username} to {github_team.name} GitHub team: {e}"
                )
                traceback.print_exc()

    def subtract_invited_members(self, members, github_team):
        """Subtract the invited members from the members list.

        We avoid sending duplicate invitations, even if the member might not have the correct role,
        because the invitation role cannot be easily obtained and the role can
        be corrected during the sync after the invitation is accepted anyways.
        """
        invitations = github_team.invitations()
        invited_members = set([invitation.login for invitation in invitations])
        return members - invited_members

    def remove_unlisted_members_from_main_team(self, github_team, desired_members):
        """Remove unlisted members from the Github main team."""
        current_members = {member.login for member in github_team.get_members()}
        for username in current_members - desired_members:
            self.remove_member_from_team(github_team, username)

    def add_or_update_member_to_team(
        self, github_team, username, role: Literal["member", "maintainer"]
    ):
        with log_operation(
            f"add/update {username} as a {role} to {github_team.name} GitHub team"
        ):
            user = self.g.get_user(username)
            github_team.add_membership(user, role=role)

    def remove_member_from_team(self, github_team, username):
        with log_operation(f"remove {username} from {github_team.name} GitHub team"):
            user = self.g.get_user(username)
            github_team.remove_membership(user)

    def sync_repos(
        self, github_team: Team, github_admin_team: Team, repos, remove_unlisted
    ):
        """Sync the repositories to the Github team.

        Give main team write access and admin team admin access to the repository.
        """
        github_repos = github_team.get_repos()
        github_repos_names = set([repo.full_name for repo in github_repos])

        # Remove any repositories from the Github team that are not in the team list
        # if the team want to remove unlisted repos.
        if remove_unlisted:
            for repo in github_repos_names:
                if repo not in repos:
                    log_message = f"remove {repo} from {github_team.name} Github team"
                    with log_operation(log_message):
                        github_team.remove_from_repos(repo)

        # Give team devs write access and team leads admin access to the repository
        for repo in repos:
            if repo not in github_repos_names:
                log_message = f"add {repo} to {github_team.name} Github team"
                with log_operation(log_message):
                    github_team.add_to_repos(repo)

            # Always update the team repository permissions, idempotent and cheap
            github_team.update_team_repository(repo, "push")
            github_admin_team.update_team_repository(repo, "admin")
