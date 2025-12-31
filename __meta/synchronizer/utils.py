import traceback
from contextlib import contextmanager
from functools import wraps

from colorama import Fore, Style

# Tracks whether the script finishes without errors
OK = True

# List of environments
ENVS = ["local", "dev", "staging", "prod"]


def print_section(section):
    bold("=" * 50)
    bold(f"Syncing {section}...")
    bold("=" * 50)
    print()


def bold(message):
    print(Style.BRIGHT + message)


def debug(message):
    print(Fore.LIGHTBLACK_EX + message)


def info(message):
    print(Fore.BLUE + message)


def warn(message):
    print(Fore.YELLOW + message)


def error(message):
    global OK
    OK = False
    print(OK)
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
    def decorator(func):
        """Decorate a team sync function to log around it.

        Team should always be the second argument of the team sync function.
        """

        @wraps(func)
        def wrapper(*args, **kwargs):
            bold(f"Syncing team {args[1]['name']}...")
            result = func(*args, **kwargs)
            print()
            return result

        return wrapper

    return decorator
