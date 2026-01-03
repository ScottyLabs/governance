import traceback
from collections.abc import Callable, Generator
from contextlib import contextmanager
from functools import wraps
from typing import ParamSpec, TypeVar

from synchronizer.models.team import Team

from .singleton import AppLoggerSingleton

P = ParamSpec("P")
R = TypeVar("R")


def print_section(section: str) -> None:
    logger = AppLoggerSingleton().logger
    logger.print("=" * 50)
    logger.print("Syncing %s...", section)
    logger.print("=" * 50 + "\n")


@contextmanager
def log_operation(operation_name: str) -> Generator[None, None, None]:
    """Context manager to log when an operation starts, finishes, or fails."""
    logger = AppLoggerSingleton().logger
    logger.info("Starting to %s...", operation_name)
    try:
        yield
        logger.success("Successfully %s.\n", operation_name)
    except Exception:
        logger.exception("Failed to %s", operation_name)
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
                # Raise an error here since this is purely a programming error
                msg = "Second argument must be a Team"
                raise TypeError(msg)

            logger = AppLoggerSingleton().logger
            logger.print("Syncing team %s...\n", team.name)
            result = func(*args, **kwargs)
            logger.print("")
            return result

        return wrapper

    return decorator
