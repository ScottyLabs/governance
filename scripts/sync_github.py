import traceback
from github import Github
from github import Auth
import os
import dotenv

dotenv.load_dotenv()


class GithubManager:
    # Initialize the GithubManager with GitHub org
    def __init__(self, contributors, teams):
        print("Initializing GithubManager")
        self.contributors = contributors
        self.teams = teams
        self.g = Github(auth=Auth.Token(os.getenv("SYNC_GITHUB_TOKEN")))
        self.org = self.g.get_organization("ScottyLabs")

    def sync(self):
        print("Syncing Github")
        self.sync_contributors()
        for _, team in self.teams.items():
            self.sync_team(team)
        print("Github sync complete")

    # Sync contributors to the GitHub organization
    def sync_contributors(self):
        # Get all existing members
        self.existing_members = set(member.login for member in self.org.get_members())
        print(f"There are {len(self.existing_members)} existing members")

        # Invite new contributors to the GitHub organization
        for github_username, _ in self.contributors.items():
            if github_username not in self.existing_members:
                print(f"Adding {github_username} to GitHub organization")
                user = self.g.get_user(github_username)
                self.org.invite_user(user=user, role="direct_member")

    # Sync the team leads and members to the Github team
    def sync_team(self, team):
        try:
            team_name = team["name"]
            print(f"Syncing team {team_name}")

            # Get or create the team and the admin team
            github_team = self.get_or_create_team(team_name)
            admin_team_name = f"{team_name} Admins"
            github_admin_team = self.get_or_create_admin_team(
                github_team, admin_team_name
            )

            # Sync the team leads to the GitHub admin team
            leads = set(team["leads"])
            self.sync_github_team(github_admin_team, leads)

            # There is no easy way to get direct members of a team,
            # so we will add leads even if they might already be in the team.
            for lead in leads:
                self.add_member_to_team(github_team, lead)

            # Sync the devs to the GitHub main team
            devs = set(team["devs"])
            self.sync_github_team(github_team, leads.union(devs))

            # Sync the repositories to the Github team
            repos = set(team["repos"])
            self.sync_repos(github_team, github_admin_team, repos)
        except Exception as e:
            print(f"Error syncing team {team['name']}: {e}")
            traceback.print_exc()

    # Get or create the Github main team, which is a subteam of the main team
    def get_or_create_team(self, team_name):
        try:
            team_slug = self.get_team_slug(team_name)
            return self.org.get_team_by_slug(team_slug)
        except Exception:
            print(f"Creating {team_name} GitHub team")
            return self.org.create_team(name=team_name, privacy="closed")

    # Get or create the Github admin team
    def get_or_create_admin_team(self, github_team, admin_team_name):
        try:
            team_slug = self.get_team_slug(admin_team_name)
            return self.org.get_team_by_slug(team_slug)
        except Exception:
            print(f"Creating {admin_team_name} GitHub team")
            return self.org.create_team(
                name=admin_team_name,
                parent_team_id=github_team.id,
                privacy="closed",
            )

    # https://docs.github.com/en/rest/teams/teams?apiVersion=2022-11-28#get-a-team-by-name
    def get_team_slug(self, team_name):
        return team_name.replace(" ", "-").lower()

    # Sync the team members to the Github team
    def sync_github_team(self, github_team, desired_members: set[str]):
        current_members = {member.login for member in github_team.get_members()}
        # --- Add new members ---
        for username in desired_members - current_members:
            self.add_member_to_team(github_team, username)

        # --- Remove extra members ---
        for username in current_members - desired_members:
            self.remove_member_from_team(github_team, username)

    def add_member_to_team(self, github_team, username):
        print(f"Adding {username} to the {github_team.name} GitHub team")
        user = self.g.get_user(username)
        github_team.add_membership(user, role="member")

    def remove_member_from_team(self, github_team, username):
        print(f"Removing {username} from the {github_team.name} GitHub team")
        user = self.g.get_user(username)
        github_team.remove_membership(user)

    # Sync the repositories to the Github team
    # Give team devs write access and team leads admin access
    def sync_repos(self, github_team, github_admin_team, repos):
        github_repos = github_team.get_repos()
        github_repos_names = set([repo.full_name for repo in github_repos])

        # Remove any repositories from the Github team that are not in the team list
        for repo in github_repos_names:
            if repo not in repos:
                print(f"Removing {repo} from {github_team.name} Github team")
                github_team.remove_from_repos(repo)

        # Give team devs write access and team leads admin access to the repository
        for repo in repos:
            github_team.add_to_repos(repo)
            github_team.update_team_repository(repo, "push")
            github_admin_team.update_team_repository(repo, "admin")
