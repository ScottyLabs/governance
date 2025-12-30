import argparse
import os

import tomllib
from colorama import init
from dotenv import load_dotenv
from sync_github import GithubManager
from sync_keycloak import KeycloakManager
from sync_slack import SlackManager
from sync_vault import VaultManager
from utils import info

load_dotenv()

# Ensures reset after each print
init(autoreset=True, strip=False)


class SyncManager:
    def __init__(self):
        info("Initializing SyncManager...\n")
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
        self.sync_github()
        self.sync_keycloak()
        self.sync_vault()
        self.sync_slack()

    def sync_github(self):
        GithubManager(self.contributors, self.teams).sync()

    def sync_keycloak(self):
        KeycloakManager(self.contributors, self.teams).sync()

    def sync_vault(self):
        VaultManager(self.teams).sync()

    def sync_slack(self):
        SlackManager(self.contributors, self.teams).sync()


def args_parser():
    parser = argparse.ArgumentParser(
        description="Sync the teams and members from the contributors and teams directories to ScottyLabs services."
    )
    parser.add_argument(
        "--services",
        nargs="+",
        choices=["github", "keycloak", "vault", "slack"],
        default=["github", "keycloak", "vault", "slack"],
        metavar="SERVICE",
        help=(
            "One or more services to sync "
            "(choices: github, keycloak, vault, slack). "
            "Defaults to syncing all."
        ),
    )
    return parser


if __name__ == "__main__":
    # Parse the arguments
    parser = args_parser()
    args = parser.parse_args()

    # Initialize the sync manager
    sync_manager = SyncManager()
    service_name_to_function = {
        "github": sync_manager.sync_github,
        "keycloak": sync_manager.sync_keycloak,
        "vault": sync_manager.sync_vault,
        "slack": sync_manager.sync_slack,
    }

    # Sync the services
    for service_name in args.services:
        service_name_to_function[service_name]()
