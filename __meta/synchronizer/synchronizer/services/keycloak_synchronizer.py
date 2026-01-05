from keycloak import KeycloakGetError

from synchronizer.clients import get_keycloak_client
from synchronizer.logger import (
    get_app_logger,
    log_operation,
    log_team_sync,
    print_section,
)
from synchronizer.models import Contributor, Team
from synchronizer.utils import (
    ENVS,
    ENVS_LITERAL,
    get_dev_server_url,
    get_local_server_url,
    get_prod_server_url,
    get_staging_server_url,
)

from .abstract_synchronizer import AbstractSynchronizer


class KeycloakSynchronizer(AbstractSynchronizer):
    ADMIN_SUFFIX = "-admins"
    EXTERNAL_ADMIN_SUFFIX = "-ext-admins"
    MEMBER_SUFFIX = "-devs"
    APPLICANT_SUFFIX = "-applicants"
    SERVICE_ACCOUNT_PREFIX = "service-account-"

    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        self.contributors = contributors
        self.teams = teams
        self.keycloak_admin = get_keycloak_client()
        self.existing_clients = [
            c["clientId"] for c in self.keycloak_admin.get_clients()
        ]
        self.logger = get_app_logger()

    def sync(self) -> None:
        print_section("Keycloak")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        # Create the OIDC clients for the team if the team wants
        if team.create_oidc_clients:
            self.create_clients(team.slug, team)
        else:
            self.logger.debug(
                "Team %s opted out of creating OIDC clients, skipping...\n",
                team.name,
            )

        # Sync the team maintainers and service accounts to the Keycloak admins group
        admin_group_name = f"{team.slug}{self.ADMIN_SUFFIX}"
        admin_usernames = self.get_usernames(team.maintainers)
        # Add the service accounts to the admins only if the OIDC clients are created
        if team.create_oidc_clients:
            admin_usernames = admin_usernames.union(
                self.get_service_account_usernames(team.slug),
            )
        self.sync_group(
            team.name,
            admin_group_name,
            admin_usernames,
            remove_unlisted=team.remove_unlisted,
        )

        # Sync team contributors who are not maintainers to Keycloak devs group
        dev_group_name = f"{team.slug}{self.MEMBER_SUFFIX}"
        devs = list(set(team.contributors) - set(team.maintainers))
        dev_usernames = self.get_usernames(devs)
        self.sync_group(
            team.name,
            dev_group_name,
            dev_usernames,
            remove_unlisted=team.remove_unlisted,
        )

        # Sync team admins to Keycloak admins group
        if team.ext_admins is not None:
            admin_group_name = f"{team.slug}{self.EXTERNAL_ADMIN_SUFFIX}"
            admins_usernames = set(team.ext_admins)
            self.sync_group(
                team.name,
                admin_group_name,
                admins_usernames,
                remove_unlisted=team.remove_unlisted,
            )

        if team.applicants is not None:
            applicant_group_name = f"{team.slug}{self.APPLICANT_SUFFIX}"
            applicants_usernames = self.get_usernames(team.applicants)
            self.sync_group(
                team.name,
                applicant_group_name,
                applicants_usernames,
                remove_unlisted=team.remove_unlisted,
            )

    def create_clients(self, team_slug: str, team: Team) -> None:
        # JSON schema should guarantee that website-slug is not None
        # when create-oidc-clients is true
        if team.website_slug is None:
            self.logger.error(
                "Website slug is not set for team %s",
                team_slug,
            )
            return

        for env in ENVS:
            client_id = f"{team_slug}-{env}"
            if client_id in self.existing_clients:
                self.logger.debug(
                    "Keycloak client %s already exists, skipping...\n",
                    client_id,
                )
                continue

            with log_operation(f"create Keycloak client {client_id}"):
                self.create_client(client_id, team.website_slug, env)

    def create_client(
        self, client_id: str, website_slug: str, env: ENVS_LITERAL
    ) -> None:
        # Generate the URIs for the client
        root_url = None
        match env:
            case "dev":
                root_url = f"https://{website_slug}.slabs-dev.org"
                server_url = get_dev_server_url(website_slug)
            case "staging":
                root_url = f"https://{website_slug}.slabs-staging.org"
                server_url = get_staging_server_url(website_slug)
            case "prod":
                root_url = f"https://{website_slug}.scottylabs.org"
                server_url = get_prod_server_url(website_slug)

        if env == "local":
            redirect_uris = [f"{get_local_server_url()}/auth/callback"]
            post_logout_redirect_uris = "http://localhost:3000/*"
        else:
            redirect_uris = [f"{server_url}/auth/callback"]
            # Permit any post-logout redirect URI with the same origin
            post_logout_redirect_uris = "/*"

        # Create the client
        self.keycloak_admin.create_client(
            payload={
                "clientId": client_id,
                "rootUrl": root_url,
                "redirectUris": redirect_uris,
                # https://github.com/keycloak/keycloak/discussions/19087#discussioncomment-5338785
                "attributes": {
                    "post.logout.redirect.uris": post_logout_redirect_uris,
                },
                "serviceAccountsEnabled": True,
                "frontchannelLogout": True,
                "protocolMappers": [
                    # Add the groups claim to the token
                    {
                        "name": "groups",
                        "protocol": "openid-connect",
                        "protocolMapper": "oidc-group-membership-mapper",
                        "config": {
                            "claim.name": "groups",
                            "userinfo.token.claim": "true",
                            "id.token.claim": "true",
                            "access.token.claim": "true",
                        },
                    },
                    # Add the audience claim to the token
                    {
                        "name": "audience",
                        "protocol": "openid-connect",
                        "protocolMapper": "oidc-audience-mapper",
                        "config": {
                            "included.client.audience": client_id,
                            "access.token.claim": "true",
                            "token.introspection.claim": "true",
                        },
                    },
                ],
            },
        )

    def get_usernames(self, members: list[str]) -> set[str]:
        usernames = set()
        for member in members:
            # Validation check guarantees that members will always be a contributor
            andrew_id = self.contributors[member].andrew_id
            if andrew_id is not None:
                # The andrew id is the username in Keycloak
                usernames.add(andrew_id)
        return usernames

    def get_service_account_usernames(self, team_slug: str) -> set[str]:
        # The service account usernames should all follow the pattern
        # "service-account-<team-slug>-<env>" (e.g. "service-account-graph-local")
        return {f"{self.SERVICE_ACCOUNT_PREFIX}{team_slug}-{env}" for env in ENVS}

    def sync_group(
        self,
        team_name: str,
        group_path: str,
        target_usernames: set[str],
        *,
        remove_unlisted: bool,
    ) -> None:
        # Get the group id and name
        group = self.get_or_create_group(group_path)
        if group is None:
            return

        group_id = group["id"]
        group_name = group["name"]

        # Get the usernames of the current members in the Keycloak group
        members = self.keycloak_admin.get_group_members(group_id)
        current_usernames = {m["username"].lower() for m in members}

        # Calculate new members
        new_member_usernames = target_usernames - current_usernames
        self.logger.debug(
            "Found %d new members for the %s group.\n",
            len(new_member_usernames),
            group_name,
        )

        # Add missing users
        for username in new_member_usernames:
            user_id = self.get_user_id_by_username(username)
            if user_id is None:
                continue

            with log_operation(f"add {username} to Keycloak group {group_name}"):
                self.keycloak_admin.group_user_add(user_id, group_id)

        # Remove extra users if the team want to remove unlisted members
        if not remove_unlisted:
            self.logger.debug(
                "Team %s opted out of removing unlisted members, skipping...\n",
                team_name,
            )
            return

        # Calculate unlisted members
        unlisted_member_usernames = current_usernames - target_usernames
        self.logger.debug(
            "Found %d unlisted members for the %s group.\n",
            len(unlisted_member_usernames),
            group_name,
        )

        # Remove unlisted members
        for username in unlisted_member_usernames:
            log_message = f"remove {username} from Keycloak group {group_name}"
            user_id = self.get_user_id_by_username(username)
            if user_id is None:
                continue

            with log_operation(log_message):
                self.keycloak_admin.group_user_remove(user_id, group_id)

    def get_or_create_group(self, group_path: str) -> dict | None:
        try:
            return self.keycloak_admin.get_group_by_path(group_path)
        except KeycloakGetError:
            with log_operation(f"create Keycloak group {group_path}"):
                self.keycloak_admin.create_group(payload={"name": group_path})
                return self.keycloak_admin.get_group_by_path(group_path)
        except Exception:
            msg = f"Error getting {group_path} Keycloak group"
            self.logger.exception(msg)
            return None

    def get_user_id_by_username(self, username: str) -> str | None:
        users = self.keycloak_admin.get_users(
            query={"username": username, "exact": True},
        )

        if not users:
            self.logger.error(
                "User %s not found in Keycloak!\n",
                username,
            )
            return None

        # Used `exact` = True, so this technically should never happen
        if len(users) > 1:
            self.logger.error(
                "Multiple users found for %s: %s!\n",
                username,
                users,
            )
            return None

        return users[0]["id"]
