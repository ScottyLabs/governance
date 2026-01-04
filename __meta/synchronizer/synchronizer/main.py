import argparse
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, Any

from dotenv import load_dotenv

from synchronizer.logger import LogStatusFilter, get_app_logger
from synchronizer.models import Contributor, Team
from synchronizer.services.codeowners_synchronizer import CodeownersSynchronizer
from synchronizer.services.github_synchronizer import GithubSynchronizer
from synchronizer.services.keycloak_synchronizer import KeycloakSynchronizer
from synchronizer.services.leadership_synchronizer import LeadershipSynchronizer
from synchronizer.services.secrets_synchronizer import SecretsSynchronizer
from synchronizer.services.slack_synchronizer import SlackSynchronizer
from synchronizer.services.vault_synchronizer import VaultSynchronizer

if TYPE_CHECKING:
    from collections.abc import Callable

load_dotenv()


class SyncManager:
    def __init__(self) -> None:
        logger = get_app_logger()
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
        GithubSynchronizer(self.contributors, self.teams).sync()

    def sync_keycloak(self) -> None:
        KeycloakSynchronizer(self.contributors, self.teams).sync()

    def sync_vault(self) -> None:
        VaultSynchronizer(self.teams).sync()

    def sync_slack(self) -> None:
        SlackSynchronizer(self.contributors, self.teams).sync()

    def sync_secrets(self) -> None:
        SecretsSynchronizer(self.teams).sync()

    def sync_codeowners(self) -> None:
        CodeownersSynchronizer(self.teams).sync()

    def sync_leadership(self) -> None:
        LeadershipSynchronizer(self.teams).sync()


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
    logger = get_app_logger()
    log_status_filter = next(
        (f for f in logger.filters if isinstance(f, LogStatusFilter)),
        None,
    )

    if log_status_filter is None:
        msg = "No LogStatusFilter found — cannot verify log state."
        logger.critical(msg)
        raise RuntimeError(msg)

    if log_status_filter.had_error:
        msg = "One or more errors were logged. Check logs for details."
        logger.critical(msg)
        raise RuntimeError(msg)

    if log_status_filter.had_warning:
        msg = "One or more warnings were logged. Check logs for details."
        logger.warning(msg)


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
        "leadership": sync_manager.sync_leadership,
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
