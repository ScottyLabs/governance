import logging
from functools import lru_cache
from typing import cast

from colorama import init

from .components import AppLogger, ColorFormatter, LogStatusFilter
from .constants import PRINT_LEVEL, SUCCESS_LEVEL


@lru_cache(maxsize=1)
def get_app_logger() -> AppLogger:
    """Get the app logger. Cache the result for reuse across the app."""
    # Register the success and print log levels
    logging.addLevelName(SUCCESS_LEVEL, "SUCCESS")
    logging.addLevelName(PRINT_LEVEL, "PRINT")

    # Set the custom logger class
    logging.setLoggerClass(AppLogger)

    # Colorama: ensures reset after each print and force keep ANSI for colors
    init(autoreset=True, strip=False)

    # Color handler for the logger
    handler = logging.StreamHandler()
    handler.setFormatter(ColorFormatter("[%(levelname)s] %(message)s"))

    # Create the logger
    app_logger = logging.getLogger(__name__)
    if app_logger.hasHandlers():
        app_logger.handlers.clear()
    app_logger.addHandler(handler)
    app_logger.setLevel(PRINT_LEVEL)

    # Add the filter to the logger
    app_logger.addFilter(LogStatusFilter())

    # Return the app logger
    return cast("AppLogger", app_logger)
