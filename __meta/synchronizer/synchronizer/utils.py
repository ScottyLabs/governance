import os
import traceback
from collections.abc import Callable, Generator
from contextlib import contextmanager
from functools import wraps
from typing import Literal, ParamSpec, TypeVar

from colorama import Fore, Style
from keycloak import KeycloakAdmin

from synchronizer.models.team import Team

P = ParamSpec("P")
R = TypeVar("R")

# Tracks whether the script finishes without errors
OK = True

# List of environments
ENVS_LITERAL = Literal["local", "dev", "staging", "prod"]
ENVS: list[ENVS_LITERAL] = ["local", "dev", "staging", "prod"]


def print_section(section: str) -> None:
    bold("=" * 50)
    bold(f"Syncing {section}...")
    bold("=" * 50)
    print()


def bold(message: str) -> None:
    print(Style.BRIGHT + message)


def debug(message: str) -> None:
    print(Fore.LIGHTBLACK_EX + message + "\n")


def info(message: str) -> None:
    print(Fore.BLUE + message)


def error(message: str) -> None:
    global OK  # noqa: PLW0603
    OK = False
    print(Fore.RED + message)


def success(message: str) -> None:
    print(Fore.GREEN + message)


@contextmanager
def log_operation(operation_name: str) -> Generator[None, None, None]:
    """Context manager to log when an operation starts, finishes, or fails."""
    info(f"Starting to {operation_name}...")
    try:
        yield
        success(f"Successfully {operation_name}.\n")
    except Exception as e:
        error(f"Failed to {operation_name}: {e}")
        traceback.print_exc()


def log_team_sync() -> Callable[[Callable[P, R]], Callable[P, R]]:
    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        """
        Decorate a team sync function to log around it.

        Team should always be the second argument of the team sync function.
        """

        @wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            team = args[1]
            if not isinstance(team, Team):
                msg = "Second argument must be a Team"
                raise TypeError(msg)

            bold(f"Syncing team {team.name}...")
            result = func(*args, **kwargs)
            print()
            return result

        return wrapper

    return decorator


def get_server_url(website_slug: str, env: ENVS_LITERAL) -> str:
    match env:
        case "local":
            return get_local_server_url()
        case "dev":
            return get_dev_server_url(website_slug)
        case "staging":
            return get_staging_server_url(website_slug)
        case "prod":
            return get_prod_server_url(website_slug)


def get_local_server_url() -> str:
    return "http://localhost"


def get_dev_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.slabs-dev.org"


def get_staging_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.slabs-staging.org"


def get_prod_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.scottylabs.org"


def get_keycloak_admin() -> KeycloakAdmin:
    realm_name = os.getenv("KEYCLOAK_REALM")
    if not realm_name:
        msg = "KEYCLOAK_REALM is not set"
        raise ValueError(msg)

    client_id = os.getenv("KEYCLOAK_CLIENT_ID")
    if not client_id:
        msg = "KEYCLOAK_CLIENT_ID is not set"
        raise ValueError(msg)

    return KeycloakAdmin(
        server_url=os.getenv("KEYCLOAK_SERVER_URL"),
        username=os.getenv("KEYCLOAK_USERNAME"),
        password=os.getenv("KEYCLOAK_PASSWORD"),
        realm_name=realm_name,
        client_id=client_id,
        user_realm_name=os.getenv("KEYCLOAK_USER_REALM"),
        verify=True,
    )
