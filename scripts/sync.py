import os
import tomllib

from scripts.sync_github import GithubManager


class SyncManager:
    def __init__(self):
        self.contributors = dict()
        self.load_contributors()

    def load_contributors(self):
        for file in os.listdir("contributors"):
            if file.endswith(".toml"):
                with open(os.path.join("contributors", file), "r") as f:
                    username = file.replace(".toml", "")
                    self.contributors[username] = tomllib.loads(f.read())

    def sync(self):
        GithubManager().sync_contributors(self.contributors)


if __name__ == "__main__":
    SyncManager().sync()
