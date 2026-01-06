import argparse
import tomllib
from pathlib import Path
from typing import Any

from dotenv import load_dotenv

from synchronizer.logger import LogStatusFilter, get_app_logger
from synchronizer.models import Contributor, Team
from synchronizer.services.abstract_synchronizer import AbstractSynchronizer
from synchronizer.services.codeowners_synchronizer import CodeownersSynchronizer
from synchronizer.services.github_synchronizer import GithubSynchronizer
from synchronizer.services.google_drive_synchronizer import GoogleDriveSynchronizer
from synchronizer.services.keycloak_synchronizer import KeycloakSynchronizer
from synchronizer.services.leadership_synchronizer import LeadershipSynchronizer
from synchronizer.services.secrets_synchronizer import SecretsSynchronizer
from synchronizer.services.slack_synchronizer import SlackSynchronizer
from synchronizer.services.vault_synchronizer import VaultSynchronizer

load_dotenv()

# Mapping of service names to their corresponding classes
SERVICE_CLASSES: dict[str, type[AbstractSynchronizer]] = {
    "github": GithubSynchronizer,
    "keycloak": KeycloakSynchronizer,
    "vault": VaultSynchronizer,
    "slack": SlackSynchronizer,
    "secrets": SecretsSynchronizer,
    "google_drive": GoogleDriveSynchronizer,
    "codeowners": CodeownersSynchronizer,
    "leadership": LeadershipSynchronizer,
}


def load_contributors() -> dict[str, Contributor]:
    """Load the contributors from the contributors directory."""
    contributors: dict[str, Contributor] = {}
    for file_path in Path("contributors").iterdir():
        if file_path.suffix == ".toml":
            with file_path.open() as f:
                username = file_path.stem
                data: dict[str, Any] = tomllib.loads(f.read())
                contributors[username] = Contributor.model_validate(data)

    return contributors


def load_teams() -> dict[str, Team]:
    """Load the teams from the teams directory."""
    teams: dict[str, Team] = {}
    for file_path in Path("teams").iterdir():
        if file_path.suffix == ".toml":
            with file_path.open() as f:
                team_name = file_path.stem
                data: dict[str, Any] = tomllib.loads(f.read())
                teams[team_name] = Team.model_validate(data)

                # Get rid of duplicates in the contributors list
                # Necessary for year-based team formatting (e.g. cmumaps.toml)
                teams[team_name].contributors = list(set(teams[team_name].contributors))

    return teams


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

    if log_status_filter.error_logged > 0:
        msg = (
            f"{log_status_filter.error_logged} error(s) were logged. "
            "Check logs for details."
        )
        logger.critical(msg)
        raise RuntimeError(msg)

    if log_status_filter.warning_logged > 0:
        msg = (
            f"{log_status_filter.warning_logged} warning(s) were logged. "
            "Check logs for details."
        )
        logger.warning(msg)


def main() -> None:
    # Parse the arguments
    services = list(SERVICE_CLASSES.keys())
    args = args_parser(services).parse_args()
    services = args.services

    # Load the contributors and teams
    logger = get_app_logger()
    logger.info("Loading contributors and teams...\n")
    contributors = load_contributors()
    teams = load_teams()

    # Sync the services
    for service_name in services:
        SERVICE_CLASSES[service_name](contributors, teams).sync()

    # Check the logger status
    check_logger_status()
