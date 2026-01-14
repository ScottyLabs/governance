import base64
import os
from typing import override

from hvac.exceptions import InvalidPath

from synchronizer.clients import get_keycloak_client
from synchronizer.clients.vault_client import get_vault_client
from synchronizer.logger import (
    log_operation,
    log_team_sync,
    print_section,
)
from synchronizer.models import Contributor, Team
from synchronizer.utils import ENVS, ENVS_LITERAL, get_server_url

from .abstract_synchronizer import AbstractSynchronizer


class SecretsSynchronizer(AbstractSynchronizer):
    MOUNT_POINT = "ScottyLabs"

    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        super().__init__(contributors, teams)

        # Initialize the clents
        self.vault_client = get_vault_client()
        self.keycloak_client = get_keycloak_client()

    @override
    def sync(self) -> None:
        print_section("Secrets")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        # Skip if the team does not want to populate secrets
        secrets_population_layout = team.secrets_population_layout
        if secrets_population_layout == "none":
            self.logger.debug(
                "Team %s opted out of secrets population, skipping...\n",
                team.name,
            )
            return

        # Skip if the team already has secrets
        if self.has_secrets(team.slug):
            self.logger.debug(
                "Team %s already has secrets, skipping...\n",
                team.slug,
            )
            return

        # Sync the secrets
        if secrets_population_layout == "single":
            # Skip if the create-oidc-clients and sync-minio flags are false for a
            # single app project since there is no secrets to populate.
            if not team.create_oidc_clients:
                self.logger.debug(
                    "There is no secrets to populate for single app project without "
                    "OIDC clients and MinIO keys, skipping team %s...\n",
                    team.slug,
                )
                return

            self.sync_single_app_secrets(team)

        elif secrets_population_layout == "multi":
            self.sync_multi_apps_secrets(
                team, create_oidc_clients=team.create_oidc_clients
            )

    def has_secrets(self, team_slug: str) -> bool:
        """Check if the team has secrets in the vault based on folder path."""
        for check in (
            # Check if the team has a folder
            self.vault_client.secrets.kv.v2.list_secrets,
            # Check if the team has a secret file
            self.vault_client.secrets.kv.v2.read_secret_metadata,
        ):
            try:
                check(path=team_slug, mount_point=self.MOUNT_POINT)
            except InvalidPath:
                continue
            except Exception:
                self.logger.exception(
                    "Failed to check secrets for %s",
                    team_slug,
                )
                return False
            else:
                return True

        return False

    def sync_single_app_secrets(self, team: Team) -> None:
        # Sync the secrets for each environment
        for env in ENVS:
            with log_operation(f"sync single-app secrets for {team.slug} {env}"):
                secret = self.get_single_app_secret(team, env)
                if secret is None:
                    continue

                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team.slug}/{env}",
                    mount_point=self.MOUNT_POINT,
                    secret=secret,
                )

    def get_single_app_secret(
        self, team: Team, env: ENVS_LITERAL
    ) -> dict[str, str] | None:
        """Include auth and MinIO secrets for the single app project."""
        secrets = {}
        auth_secrets = self.get_auth_secrets(team.slug, env)
        if auth_secrets:
            secrets.update(auth_secrets)
        if team.sync_minio:
            minio_secrets = self.get_minio_secrets(team)
            if minio_secrets:
                secrets.update(minio_secrets)
        return secrets

    def sync_multi_apps_secrets(self, team: Team, *, create_oidc_clients: bool) -> None:
        # Sync the secrets for each environment
        for env in ENVS:
            with log_operation(f"sync multi-apps secrets for {team.slug} {env}"):
                # Double check the website slug is set
                website_slug = team.website_slug
                if website_slug is None:
                    self.logger.error(
                        "Website slug is not set for team %s",
                        team.slug,
                    )
                    return

                web_secret, server_secret = self.get_multi_apps_secret(
                    team,
                    website_slug,
                    env,
                    create_oidc_clients=create_oidc_clients,
                )
                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team.slug}/{env}/web",
                    mount_point=self.MOUNT_POINT,
                    secret=web_secret,
                )
                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team.slug}/{env}/server",
                    mount_point=self.MOUNT_POINT,
                    secret=server_secret,
                )

    def get_multi_apps_secret(
        self,
        team: Team,
        website_slug: str,
        env: ENVS_LITERAL,
        *,
        create_oidc_clients: bool,
    ) -> tuple[dict[str, str], dict[str, str]]:
        """Include auth secrets for the multi apps project."""
        # Get the server url and populate the secrets
        server_url = get_server_url(website_slug, env)
        web_secret = {"VITE_SERVER_URL": server_url}
        server_secret = {"SERVER_URL": server_url}

        # Populate the Redis and Database URLs
        if env == "local":
            server_secret["REDIS_URL"] = "redis://redis:6379"
            server_secret["DATABASE_URL"] = (
                f"postgresql://postgres:donotuseinprod@postgres:5432/{team.slug}"
            )
        else:
            server_secret["REDIS_URL"] = "${{REDIS.REDIS_URL}}"
            server_secret["DATABASE_URL"] = "${{Postgres.DATABASE_URL}}"
            server_secret["RAILWAY_DOCKERFILE_PATH"] = "/apps/server/Dockerfile"

        # Allow any https prefix
        # Note that we need the
        https_origin_prefix = r"^https://([a-z0-9-]+\.)*"

        # Populate the allowed origins regex
        match env:
            case "local":
                # Allow all origins for local development
                server_secret["ALLOWED_ORIGINS_REGEX"] = r"^https?://localhost:\d{4}$"
            case "dev":
                # Allow all ScottyLabs dev subdomains and any vercel preview domains
                # (https://<team-slug>-<random 9 characters>-scottylabs.vercel.app)
                # for dev development
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{https_origin_prefix}slabs-dev\.org$,"
                    rf"^https://{team.slug}-[0-9a-z]{{9}}-scottylabs\.vercel\.app$"
                )
            case "staging":
                # Allow all ScottyLabs staging subdomains for staging development
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{https_origin_prefix}slabs-staging\.org$"
                )
            case "prod":
                # Allow all ScottyLabs production subdomains for production
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{https_origin_prefix}scottylabs\.org$"
                )

        # Populate the auth secrets if the create-oidc-clients flag is true
        if create_oidc_clients:
            auth_secrets = self.get_auth_secrets(team.slug, env)
            if auth_secrets:
                server_secret.update(auth_secrets)

        if team.sync_minio:
            minio_secrets = self.get_minio_secrets(team)
            if minio_secrets:
                server_secret.update(minio_secrets)

        return web_secret, server_secret

    def get_auth_secrets(
        self, team_slug: str, env: ENVS_LITERAL
    ) -> dict[str, str] | None:
        """Include auth secrets for the single app project."""
        # Get the client id and client secret from the Keycloak client
        client_id = f"{team_slug}-{env}"
        internal_client_id = self.keycloak_client.get_client_id(client_id)
        if internal_client_id is None:
            self.logger.error(
                "Client %s not found in Keycloak",
                client_id,
            )
            return None

        client_secrets = self.keycloak_client.get_client_secrets(internal_client_id)
        client_secret = client_secrets["value"]

        # Construct the issuer and JWKS uri
        issuer = (
            f"{os.getenv('KEYCLOAK_SERVER_URL')}/realms/{os.getenv('KEYCLOAK_REALM')}"
        )
        jwks_uri = f"{issuer}/protocol/openid-connect/certs"

        # Generate a random 48-byte string and encode it as base64 (64 characters)
        # for the auth session secret
        auth_session_secret = base64.b64encode(os.urandom(48)).decode("utf-8")

        return {
            "AUTH_CLIENT_ID": client_id,
            "AUTH_CLIENT_SECRET": client_secret,
            "AUTH_ISSUER": issuer,
            "AUTH_JWKS_URI": jwks_uri,
            "AUTH_SESSION_SECRET": auth_session_secret,
        }

    def get_minio_secrets(self, team: Team) -> dict[str, str] | None:
        """Include MinIO secrets for the multi apps project."""
        # MinIO secrets would always be set together in the MinIO synchronization.
        s3_endpoint = os.getenv("S3_ENDPOINT")
        if (
            team.minio_readonly_access_key is None
            or team.minio_readonly_secret_key is None
            or team.minio_admin_access_key is None
            or team.minio_admin_secret_key is None
            or s3_endpoint is None
        ):
            return None

        return {
            "S3_READONLY_ACCESS_KEY": team.minio_readonly_access_key,
            "S3_READONLY_SECRET_KEY": team.minio_readonly_secret_key,
            "S3_ADMIN_ACCESS_KEY": team.minio_admin_access_key,
            "S3_ADMIN_SECRET_KEY": team.minio_admin_secret_key,
            "S3_ENDPOINT": s3_endpoint,
        }
