import argparse
import os
import tomllib

from colorama import init
from dotenv import load_dotenv

import synchronizer.utils
from synchronizer.sync_github import GithubManager
from synchronizer.sync_keycloak import KeycloakManager
from synchronizer.sync_secrets import SecretsManager
from synchronizer.sync_slack import SlackManager
from synchronizer.sync_vault import VaultManager
from synchronizer.utils import error, info

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
                with open(os.path.join("contributors", file)) as f:
                    username = file.replace(".toml", "")
                    self.contributors[username] = tomllib.loads(f.read())

    def load_teams(self):
        for file in os.listdir("teams"):
            if file.endswith(".toml"):
                with open(os.path.join("teams", file)) as f:
                    team_name = file.replace(".toml", "")
                    self.teams[team_name] = tomllib.loads(f.read())

    def sync_github(self):
        GithubManager(self.contributors, self.teams).sync()

    def sync_keycloak(self):
        KeycloakManager(self.contributors, self.teams).sync()

    def sync_vault(self):
        VaultManager(self.teams).sync()

    def sync_slack(self):
        SlackManager(self.contributors, self.teams).sync()

    def sync_secrets(self):
        SecretsManager(self.teams).sync()


def args_parser():
    parser = argparse.ArgumentParser(
        description="Sync the teams and members from the contributors and teams directories to ScottyLabs services.",
    )
    parser.add_argument(
        "--services",
        nargs="+",
        choices=["github", "keycloak", "vault", "slack", "secrets"],
        default=["github", "keycloak", "vault", "slack", "secrets"],
        metavar="SERVICE",
        help=(
            "One or more services to sync "
            "(choices: github, keycloak, vault, slack, secrets). "
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
        "secrets": sync_manager.sync_secrets,
    }

    # Sync the services
    for service_name in args.services:
        service_name_to_function[service_name]()

    # Exit with code 1 if any error occured
    if not synchronizer.utils.OK:
        error("One or more services failed to sync. Check the logs for details.")
        exit(1)
