import argparse
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast

from colorama import init
from dotenv import load_dotenv

import synchronizer.utils
from synchronizer.sync_github import GithubManager
from synchronizer.sync_keycloak import KeycloakManager
from synchronizer.sync_secrets import SecretsManager
from synchronizer.sync_slack import SlackManager
from synchronizer.sync_vault import VaultManager
from synchronizer.utils import error, info

if TYPE_CHECKING:
    from synchronizer.models.contributor import Contributor
    from synchronizer.models.team import Team

load_dotenv()

# Ensures reset after each print
init(autoreset=True, strip=False)


class SyncManager:
    def __init__(self) -> None:
        info("Initializing SyncManager...\n")
        self.contributors: dict[str, Contributor] = {}
        self.load_contributors()

        self.teams: dict[str, Team] = {}
        self.load_teams()

    def load_contributors(self) -> None:
        for file_path in Path("../../contributors").iterdir():
            if file_path.suffix == ".toml":
                with file_path.open() as f:
                    username = file_path.stem
                    data: dict[str, Any] = tomllib.loads(f.read())
                    self.contributors[username] = cast("Contributor", data)

    def load_teams(self) -> None:
        for file_path in Path("../../teams").iterdir():
            if file_path.suffix == ".toml":
                with file_path.open() as f:
                    team_name = file_path.stem
                    data: dict[str, Any] = tomllib.loads(f.read())
                    self.teams[team_name] = cast("Team", data)

    def sync_github(self) -> None:
        GithubManager(self.contributors, self.teams).sync()

    def sync_keycloak(self) -> None:
        KeycloakManager(self.contributors, self.teams).sync()

    def sync_vault(self) -> None:
        VaultManager(self.teams).sync()

    def sync_slack(self) -> None:
        SlackManager(self.contributors, self.teams).sync()

    def sync_secrets(self) -> None:
        SecretsManager(self.teams).sync()


def args_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Sync the teams and members from the contributors and teams directories "
            "to ScottyLabs services."
        ),
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
        sys.exit(1)
