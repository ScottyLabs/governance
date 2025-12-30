import traceback
from contextlib import contextmanager
from functools import wraps

from colorama import Fore, Style

ENVS = ["local", "dev", "staging", "prod"]


def print_section(section):
    print(Style.BRIGHT + "=" * 50)
    print(Style.BRIGHT + f"Syncing {section}...")
    print(Style.BRIGHT + "=" * 50)
    print()


def debug(message):
    print(Style.DIM + message)


def info(message):
    print(Fore.BLUE + message)


def warn(message):
    print(Fore.YELLOW + message)


def error(message):
    print(Fore.RED + message)


def success(message):
    print(Fore.GREEN + message)


@contextmanager
def log_operation(operation_name):
    """Context manager to log when an operation starts, finishes, or fails."""
    info(f"Starting to {operation_name}...")
    try:
        yield
        success(f"Successfully {operation_name}.\n")
    except Exception as e:
        error(f"Failed to {operation_name}: {e}")
        traceback.print_exc()


def log_team_sync():
    """Decorator to log around a team sync. Team should always be the second argument."""

    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # team is always the second argument
            print(f"Syncing team {args[1]['name']}...")
            result = func(*args, **kwargs)
            print()
            return result

        return wrapper

    return decorator
