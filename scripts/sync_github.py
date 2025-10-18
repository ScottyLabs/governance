from github import Github
from github import Auth
import os
import dotenv

dotenv.load_dotenv()


class GithubManager:
    def __init__(self):
        print("Initializing GithubManager")
        self.auth = Auth.Token(os.getenv("GITHUB_TOKEN"))
        self.g = Github(auth=self.auth)
        self.org = self.g.get_organization("ScottyLabs")
        self.existing_members = set(member.login for member in self.org.get_members())
        print(
            f"GithubManager initialized with {len(self.existing_members)} existing members"
        )

    def sync_contributors(self, contributors):
        for github_username, contributor in contributors.items():
            if github_username not in self.existing_members:
                print(f"Adding {github_username} to GitHub organization")
                user = self.g.get_user(github_username)
                self.org.invite_user(user=user, role="direct_member")
