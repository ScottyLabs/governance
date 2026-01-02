import traceback
from collections.abc import Callable, Generator
from contextlib import contextmanager
from functools import wraps
from typing import ParamSpec, TypeVar

from colorama import Fore, Style

from synchronizer.models.team import Team

P = ParamSpec("P")
R = TypeVar("R")

# Tracks whether the script finishes without errors
OK = True


def print_section(section: str) -> None:
    bold("=" * 50)
    bold(f"Syncing {section}...")
    bold("=" * 50)
    print()


def bold(message: str) -> None:
    print(Style.BRIGHT + message)


def debug(message: str, *, new_line: bool = True) -> None:
    msg = message + "\n" if new_line else message
    print(Fore.LIGHTBLACK_EX + msg)


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
