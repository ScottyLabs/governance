import os
from functools import lru_cache

import hvac

from synchronizer.logger import get_app_logger

VAULT_URL = "https://secrets.scottylabs.org"


@lru_cache(maxsize=1)
def get_vault_client() -> hvac.Client:
    """Get the Vault client. Cache the result for reuse across the app."""
    logger = get_app_logger()

    vault_token = os.getenv("VAULT_TOKEN")
    if not vault_token:
        msg = "VAULT_TOKEN is not set"
        logger.critical(msg)
        raise RuntimeError(msg)

    return hvac.Client(url=VAULT_URL, token=vault_token)
