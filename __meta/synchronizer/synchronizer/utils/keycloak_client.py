import os
import sys
from typing import Optional

from keycloak import KeycloakAdmin

from synchronizer.utils.logging import get_logger


class KeycloakClient:
    """Singleton wrapper around KeycloakAdmin for reuse across the app."""

    _instance: Optional["KeycloakClient"] = None
    _admin: KeycloakAdmin | None = None

    def __new__(cls) -> "KeycloakClient":
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.init_admin()
        return cls._instance

    def init_admin(self) -> None:
        """Initialize the Keycloak admin client once."""
        realm_name = os.getenv("KEYCLOAK_REALM")
        self.logger = get_logger()

        if not realm_name:
            msg = "KEYCLOAK_REALM is not set"
            self.logger.critical(msg)
            sys.exit(1)

        client_id = os.getenv("KEYCLOAK_CLIENT_ID")
        if not client_id:
            msg = "KEYCLOAK_CLIENT_ID is not set"
            self.logger.critical(msg)
            sys.exit(1)

        self._admin = KeycloakAdmin(
            server_url=os.getenv("KEYCLOAK_SERVER_URL"),
            username=os.getenv("KEYCLOAK_USERNAME"),
            password=os.getenv("KEYCLOAK_PASSWORD"),
            realm_name=realm_name,
            client_id=client_id,
            user_realm_name=os.getenv("KEYCLOAK_USER_REALM"),
            verify=True,
        )

    @property
    def admin(self) -> KeycloakAdmin:
        if self._admin is None:
            msg = "Keycloak admin client not initialized"
            raise ValueError(msg)
        return self._admin
