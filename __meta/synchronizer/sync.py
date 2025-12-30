import os
import tomllib
from utils import info

from sync_github import GithubManager
from sync_keycloak import KeycloakManager
from sync_vault import VaultManager
from sync_slack import SlackManager

from dotenv import load_dotenv

load_dotenv()


class SyncManager:
    def __init__(self):
        info("Initializing SyncManager...")
        self.contributors = dict()
        self.load_contributors()

        self.teams = dict()
        self.load_teams()

    def load_contributors(self):
        for file in os.listdir("contributors"):
            if file.endswith(".toml"):
                with open(os.path.join("contributors", file), "r") as f:
                    username = file.replace(".toml", "")
                    self.contributors[username] = tomllib.loads(f.read())

    def load_teams(self):
        for file in os.listdir("teams"):
            if file.endswith(".toml"):
                with open(os.path.join("teams", file), "r") as f:
                    team_name = file.replace(".toml", "")
                    self.teams[team_name] = tomllib.loads(f.read())

    def sync(self):
        GithubManager(self.contributors, self.teams).sync()
        KeycloakManager(self.contributors, self.teams).sync()
        VaultManager(self.teams).sync()
        SlackManager(self.contributors, self.teams).sync()


if __name__ == "__main__":
    SyncManager().sync()
