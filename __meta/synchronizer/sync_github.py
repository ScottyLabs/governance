import os

from github import Auth, Github
from utils import debug, log_operation, log_team_sync, print_section


class GithubManager:
    ADMIN_SUFFIX = " Admins"

    def __init__(self, contributors, teams):
        """Initialize the GithubManager with GitHub org."""
        self.contributors = contributors
        self.teams = teams
        self.g = Github(auth=Auth.Token(os.getenv("SYNC_GITHUB_TOKEN")))
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
                    self.org.invite_user(user=user, role="direct_member")

    @log_team_sync()
    def sync_team(self, team):
        """Sync the team to the GitHub organization."""
        team_name = team["name"]

        # Get or create the team and the admin team
        github_team = self.get_or_create_team(team_name)
        admin_team_name = f"{team_name}{self.ADMIN_SUFFIX}"
        github_admin_team = self.get_or_create_admin_team(github_team, admin_team_name)

        # Sync the team leads to the GitHub main team and admin team
        leads = set(team["leads"])
        self.sync_github_admin_team(github_admin_team, github_team, leads)

        # Sync the devs to the GitHub main team
        devs = set(team["devs"])
        # Need to include both leads and devs so the leads won't be removed from the main team.
        self.sync_github_main_team(github_team, leads.union(devs))

        # Sync the repositories to the Github team
        repos = set(team["repos"])
        self.sync_repos(github_team, github_admin_team, repos)

    def get_or_create_team(self, team_name):
        """Get or create the Github main team."""
        try:
            team_slug = self.get_team_slug(team_name)
            return self.org.get_team_by_slug(team_slug)
        except Exception:
            with log_operation(f"create {team_name} GitHub team"):
                return self.org.create_team(name=team_name, privacy="closed")

    def get_or_create_admin_team(self, github_team, admin_team_name):
        """Get or create the Github admin team, which is a subteam of the main team."""
        try:
            team_slug = self.get_team_slug(admin_team_name)
            return self.org.get_team_by_slug(team_slug)
        except Exception:
            with log_operation(f"create {admin_team_name} GitHub team"):
                return self.org.create_team(
                    name=admin_team_name,
                    parent_team_id=github_team.id,
                    # We can have all teams visible to all members of the organization.
                    privacy="closed",  # one of "secret" | "closed"
                )

    # https://docs.github.com/en/rest/teams/teams?apiVersion=2022-11-28#get-a-team-by-name
    def get_team_slug(self, team_name):
        return team_name.replace(" ", "-").lower()

    def sync_github_main_team(self, github_team, desired_members):
        """Sync the team members to the Github main team."""
        current_members = {member.login for member in github_team.get_members()}
        # Add new members
        for username in desired_members - current_members:
            self.add_member_to_team(github_team, username)

        # Remove extra members
        for username in current_members - desired_members:
            self.remove_member_from_team(github_team, username)

    def sync_github_admin_team(self, github_admin_team, github_team, desired_members):
        """Sync the team leads to the GitHub main team and admin team."""
        current_members = {member.login for member in github_admin_team.get_members()}
        # Add new members
        for username in desired_members - current_members:
            # Members of the admin subteam team are also members of the main team
            # but doesn't show up in GitHub UI, so we have to explicitly add
            # the leads to the main team here.
            self.add_member_to_team(github_team, username)
            self.add_member_to_team(github_admin_team, username)

        # Remove extra members
        for username in current_members - desired_members:
            self.remove_member_from_team(github_admin_team, username)

    def add_member_to_team(self, github_team, username):
        with log_operation(f"add {username} to {github_team.name} GitHub team"):
            user = self.g.get_user(username)
            github_team.add_membership(user, role="member")

    def remove_member_from_team(self, github_team, username):
        with log_operation(f"remove {username} from {github_team.name} GitHub team"):
            user = self.g.get_user(username)
            github_team.remove_membership(user)

    def sync_repos(self, github_team, github_admin_team, repos):
        """Sync the repositories to the Github team.

        Give main team write access and admin team admin access to the repository.
        """
        github_repos = github_team.get_repos()
        github_repos_names = set([repo.full_name for repo in github_repos])

        # Remove any repositories from the Github team that are not in the team list
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
