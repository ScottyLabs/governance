import argparse
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, Any

from dotenv import load_dotenv

from synchronizer.logger import AppLoggerSingleton, LogStatusFilter
from synchronizer.models import Contributor, Team
from synchronizer.services.sync_codeowners import CodeownersManager
from synchronizer.services.sync_github import GithubManager
from synchronizer.services.sync_keycloak import KeycloakManager
from synchronizer.services.sync_secrets import SecretsManager
from synchronizer.services.sync_slack import SlackManager
from synchronizer.services.sync_vault import VaultManager

if TYPE_CHECKING:
    from collections.abc import Callable

load_dotenv()


class SyncManager:
    def __init__(self) -> None:
        logger = AppLoggerSingleton().logger
        logger.info("Initializing SyncManager...\n")
        self.contributors: dict[str, Contributor] = {}
        self.load_contributors()

        self.teams: dict[str, Team] = {}
        self.load_teams()

    def load_contributors(self) -> None:
        for file_path in Path("contributors").iterdir():
            if file_path.suffix == ".toml":
                with file_path.open() as f:
                    username = file_path.stem
                    data: dict[str, Any] = tomllib.loads(f.read())
                    self.contributors[username] = Contributor.model_validate(data)

    def load_teams(self) -> None:
        for file_path in Path("teams").iterdir():
            if file_path.suffix == ".toml":
                with file_path.open() as f:
                    team_name = file_path.stem
                    data: dict[str, Any] = tomllib.loads(f.read())
                    self.teams[team_name] = Team.model_validate(data)

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

    def sync_codeowners(self) -> None:
        CodeownersManager(self.teams).sync()


def args_parser(services: list[str]) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Sync the teams and members from the contributors and teams directories "
            "to ScottyLabs services."
        ),
    )
    parser.add_argument(
        "--services",
        nargs="+",
        choices=services,
        default=services,
        metavar="SERVICE",
        help=(
            "One or more services to sync "
            f"(choices: {', '.join(services)}). "
            "Defaults to syncing all."
        ),
    )
    return parser


def check_logger_status() -> None:
    """Check log filter flags and exit or warn if needed."""
    logger = AppLoggerSingleton().logger
    log_status_filter = next(
        (f for f in logger.filters if isinstance(f, LogStatusFilter)),
        None,
    )

    if log_status_filter is None:
        logger.critical("No LogStatusFilter found — cannot verify log state.")
        sys.exit(1)

    if log_status_filter.had_error:
        logger.critical("One or more errors were logged. Check logs for details.")
        sys.exit(1)

    if log_status_filter.had_warning:
        logger.warning("One or more warnings were logged. Check logs for details.")


def main() -> None:
    # Initialize the sync manager
    sync_manager = SyncManager()
    service_name_to_function: dict[str, Callable[[], None]] = {
        "github": sync_manager.sync_github,
        "keycloak": sync_manager.sync_keycloak,
        "vault": sync_manager.sync_vault,
        "slack": sync_manager.sync_slack,
        "secrets": sync_manager.sync_secrets,
        "codeowners": sync_manager.sync_codeowners,
    }

    services = list(service_name_to_function.keys())

    # Parse the arguments
    parser = args_parser(services)
    args = parser.parse_args()

    # Sync the services
    for service_name in args.services:
        service_name_to_function[service_name]()

    # Check the logger status
    check_logger_status()
