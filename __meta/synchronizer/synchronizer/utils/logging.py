import logging
import traceback
from collections.abc import Callable, Generator
from contextlib import contextmanager
from functools import wraps
from typing import Any, ClassVar, ParamSpec, TypeVar, cast

from colorama import Fore, Style, init
from colorama.ansi import Back

from synchronizer.models.team import Team

P = ParamSpec("P")
R = TypeVar("R")

# Register the success log level
SUCCESS_LEVEL = 25
logging.addLevelName(SUCCESS_LEVEL, "SUCCESS")


# Custom logger class that adds a .success() method
class AppLogger(logging.Logger):
    """Custom logger that adds a .success() method."""

    def success(self, msg: str, *args: Any, **kwargs: Any) -> None:  # noqa: ANN401
        if self.isEnabledFor(SUCCESS_LEVEL):
            self._log(SUCCESS_LEVEL, msg, args, **kwargs)


# Set the custom logger class
logging.setLoggerClass(AppLogger)


# A logger filter to track if an error occurred
class ErrorFlagFilter(logging.Filter):
    def __init__(self) -> None:
        super().__init__()
        self.had_error = False

    def filter(self, record: logging.LogRecord) -> bool:
        if record.levelno >= logging.ERROR:
            self.had_error = True
        return True


# A global logger instance
_LOGGER: AppLogger | None = None


def setup_logging() -> None:
    # Colorama: ensures reset after each print and force keep ANSI for colors
    init(autoreset=True, strip=False)

    # Color formatter for the logger
    class ColorFormatter(logging.Formatter):
        COLOR_MAP: ClassVar = {
            logging.DEBUG: Fore.LIGHTBLACK_EX,
            logging.INFO: Fore.BLUE,
            SUCCESS_LEVEL: Fore.GREEN,
            logging.WARNING: Fore.YELLOW,
            logging.ERROR: Fore.RED,
            logging.CRITICAL: Back.RED + Fore.WHITE,
        }

        def format(self, record: logging.LogRecord) -> str:
            color = self.COLOR_MAP.get(record.levelno, "")
            message = super().format(record)
            return f"{color}{message}{Style.RESET_ALL}"

    # Color handler for the logger
    handler = logging.StreamHandler()
    handler.setFormatter(ColorFormatter("[%(levelname)s] %(message)s"))

    # Create the logger
    app_logger = logging.getLogger(__name__)
    app_logger.addHandler(handler)
    app_logger.setLevel(logging.DEBUG)

    # Add the filter to the logger
    app_logger.addFilter(ErrorFlagFilter())

    global _LOGGER  # noqa: PLW0603
    _LOGGER = cast("AppLogger", app_logger)


def get_logger() -> AppLogger:
    if _LOGGER is None:
        msg = "Logger is not initialized"
        raise RuntimeError(msg)

    return _LOGGER


def print_section(section: str) -> None:
    bold("=" * 50)
    bold(f"Syncing {section}...")
    bold("=" * 50)
    print()  # noqa: T201


def bold(message: str) -> None:
    print(Style.BRIGHT + message)  # noqa: T201


@contextmanager
def log_operation(operation_name: str) -> Generator[None, None, None]:
    """Context manager to log when an operation starts, finishes, or fails."""
    logger = get_logger()
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
                msg = "Second argument must be a Team"
                raise TypeError(msg)

            bold(f"Syncing team {team.name}...\n")
            result = func(*args, **kwargs)
            print()  # noqa: T201
            return result

        return wrapper

    return decorator
