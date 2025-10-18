import os
import tomllib

from scripts.sync_github import GithubManager


class SyncManager:
    def __init__(self):
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


if __name__ == "__main__":
    SyncManager().sync()
