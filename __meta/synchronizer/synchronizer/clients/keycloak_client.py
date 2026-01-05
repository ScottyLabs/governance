import os
from functools import lru_cache

from keycloak import KeycloakAdmin

from synchronizer.logger import get_app_logger

# Expose the username so the word "admin" won't be filtered in GitHub Actions...
KEYCLOAK_USERNAME = "admin"


@lru_cache(maxsize=1)
def get_keycloak_client() -> KeycloakAdmin:
    """Get the Keycloak client. Cache the result for reuse across the app."""
    logger = get_app_logger()

    realm_name = os.getenv("KEYCLOAK_REALM")
    if not realm_name:
        msg = "KEYCLOAK_REALM is not set"
        logger.critical(msg)
        raise RuntimeError(msg)

    client_id = os.getenv("KEYCLOAK_CLIENT_ID")
    if not client_id:
        msg = "KEYCLOAK_CLIENT_ID is not set"
        logger.critical(msg)
        raise RuntimeError(msg)

    return KeycloakAdmin(
        server_url=os.getenv("KEYCLOAK_SERVER_URL"),
        username=KEYCLOAK_USERNAME,
        password=os.getenv("KEYCLOAK_PASSWORD"),
        realm_name=realm_name,
        client_id=client_id,
        user_realm_name=os.getenv("KEYCLOAK_USER_REALM"),
        verify=True,
    )
