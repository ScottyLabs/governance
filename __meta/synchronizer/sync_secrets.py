import base64
import os
import traceback

import hvac
from hvac.exceptions import InvalidPath
from utils import (
    ENVS,
    debug,
    error,
    get_keycloak_admin,
    get_server_url,
    log_operation,
    log_team_sync,
    print_section,
)


class SecretsManager:
    VAULT_URL = "https://secrets.scottylabs.org"
    MOUNT_POINT = "ScottyLabs"

    def __init__(self, teams):
        realm_name = os.getenv("KEYCLOAK_REALM")
        if not realm_name:
            raise ValueError("KEYCLOAK_REALM is not set")

        client_id = os.getenv("KEYCLOAK_CLIENT_ID")
        if not client_id:
            raise ValueError("KEYCLOAK_CLIENT_ID is not set")

        self.teams = teams
        self.vault_client = hvac.Client(
            url=self.VAULT_URL, token=os.getenv("VAULT_TOKEN")
        )
        self.keycloak_client = get_keycloak_admin()

    def sync(self):
        print_section("Secrets")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team):
        # Skip if the team does not want to populate secrets
        secrets_population_layout = team.get("secrets-population-layout", "multi")
        if secrets_population_layout == "none":
            return

        # Skip if the team already has secrets
        if self.has_secrets(team["slug"]):
            debug(f"Team {team['slug']} already has secrets, skipping...")
            return

        # Get the create-oidc-clients flag
        create_oidc_clients = team.get("create-oidc-clients", True)

        # Sync the secrets
        if secrets_population_layout == "single":
            # Skip if the create-oidc-clients flag is false for a single app project.
            # Probably a script project like event-scraper that does not need auto secret sync.
            if not create_oidc_clients:
                debug(
                    f"There is no secrets to populate for single app project with no OIDC clients, skipping team {team['slug']}..."
                )
                return

            self.sync_single_app_secrets(team)

        elif secrets_population_layout == "multi":
            self.sync_multi_apps_secrets(team, create_oidc_clients)

    def has_secrets(self, team_slug):
        """Check if the team has secrets in the vault based on path to team-slug folder."""
        for check in (
            # Check if the team has a folder
            self.vault_client.secrets.kv.v2.list_secrets,
            # Check if the team has a secret file
            self.vault_client.secrets.kv.v2.read_secret_metadata,
        ):
            try:
                check(path=team_slug, mount_point=self.MOUNT_POINT)
                return True
            except InvalidPath:
                continue
            except Exception as e:
                error(f"Failed to check secrets for {team_slug}: {e}")
                traceback.print_exc()
                return False

        return False

    def sync_single_app_secrets(self, team):
        # Get the team slug
        team_slug = team["slug"]

        # Sync the secrets for each environment
        for env in ENVS:
            with log_operation(f"sync single-app secrets for {team_slug} {env}"):
                secret = self.get_single_app_secret(team_slug, env)
                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team_slug}/{env}",
                    mount_point=self.MOUNT_POINT,
                    secret=secret,
                )

    def get_single_app_secret(self, team_slug, env):
        """Include only auth secrets for the single app project."""
        return self.get_auth_secrets(team_slug, env)

    def get_auth_secrets(self, team_slug, env):
        """Include auth secrets for the single app project."""
        # Get the client id and client secret from the Keycloak client
        client_id = f"{team_slug}-{env}"
        internal_client_id = self.keycloak_client.get_client_id(client_id)
        client_secrets = self.keycloak_client.get_client_secrets(internal_client_id)
        client_secret = client_secrets["value"]

        # Construct the issuer and JWKS uri
        issuer = (
            f"{os.getenv('KEYCLOAK_SERVER_URL')}/realms/{os.getenv('KEYCLOAK_REALM')}"
        )
        jwks_uri = f"{issuer}/protocol/openid-connect/certs"

        # Generate a random 48-byte string and encode it as base64 (64 characters) for the auth session secret
        auth_session_secret = base64.b64encode(os.urandom(48)).decode("utf-8")

        return {
            "AUTH_CLIENT_ID": client_id,
            "AUTH_CLIENT_SECRET": client_secret,
            "AUTH_ISSUER": issuer,
            "AUTH_JWKS_URI": jwks_uri,
            "AUTH_SESSION_SECRET": auth_session_secret,
        }

    def sync_multi_apps_secrets(self, team, create_oidc_clients):
        # Get the team slug
        team_slug = team["slug"]

        # Sync the secrets for each environment
        for env in ENVS:
            with log_operation(f"sync multi-apps secrets for {team_slug} {env}"):
                web_secret, server_secret = self.get_multi_apps_secret(
                    team_slug, env, create_oidc_clients
                )
                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team_slug}/{env}/web",
                    mount_point=self.MOUNT_POINT,
                    secret=web_secret,
                )
                self.vault_client.secrets.kv.v2.create_or_update_secret(
                    path=f"{team_slug}/{env}/server",
                    mount_point=self.MOUNT_POINT,
                    secret=server_secret,
                )

    def get_multi_apps_secret(self, team_slug, env, create_oidc_clients):
        """Include auth secrets for the multi apps project."""
        # Get the server url and populate the secrets
        server_url = get_server_url(team_slug, env)
        web_secret = {"VITE_SERVER_URL": server_url}
        server_secret = {"SERVER_URL": server_url}

        # Populate the Redis and Database URLs
        if env == "local":
            server_secret["REDIS_URL"] = "redis://redis:6379"
            server_secret["DATABASE_URL"] = (
                f"postgresql://postgres:donotuseinprod@postgres:5432/{team_slug}"
            )
        else:
            server_secret["REDIS_URL"] = "${{REDIS.REDIS_URL}}"
            server_secret["DATABASE_URL"] = "${{Postgres.DATABASE_URL}}"
            server_secret["RAILWAY_DOCKERFILE_PATH"] = "/apps/server/Dockerfile"

        # Allow any https prefix
        # Note that we need the
        HTTPS_ORIGIN_PREFIX = r"^https:\/\/([a-z0-9-]+\.)*"

        # Populate the allowed origins regex
        match env:
            case "local":
                # Allow all origins for local development
                server_secret["ALLOWED_ORIGINS_REGEX"] = r"^https?:\/\/localhost:\d{4}$"
            case "dev":
                # Allow all ScottyLabs dev subdomains for dev development and
                # any vercel preview domains (https://<team-slug>-<random-string of 9 characters>-scottylabs.vercel.app)
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{HTTPS_ORIGIN_PREFIX}slabs-dev\.org$,"
                    rf"^https:\/\/{team_slug}-[0-9a-z]{{9}}-scottylabs\.vercel\.app$"
                )
            case "staging":
                # Allow all ScottyLabs staging subdomains for staging development
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{HTTPS_ORIGIN_PREFIX}slabs-staging\.org$"
                )
            case "prod":
                # Allow all ScottyLabs production subdomains for production
                server_secret["ALLOWED_ORIGINS_REGEX"] = (
                    rf"{HTTPS_ORIGIN_PREFIX}scottylabs\.org$"
                )

        # Populate the auth secrets if the create-oidc-clients flag is true
        if create_oidc_clients:
            auth_secrets = self.get_single_app_secret(team_slug, env)
            server_secret.update(auth_secrets)

        return web_secret, server_secret
