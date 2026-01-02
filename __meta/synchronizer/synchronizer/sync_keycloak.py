from synchronizer.models.contributor import Contributor
from synchronizer.models.team import Team
from synchronizer.utils import (
    ENVS,
    ENVS_LITERAL,
    debug,
    error,
    get_dev_server_url,
    get_keycloak_admin,
    get_local_server_url,
    get_prod_server_url,
    get_staging_server_url,
    log_operation,
    log_team_sync,
    print_section,
)


class KeycloakManager:
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
        self.keycloak_admin = get_keycloak_admin()
        self.existing_clients = [
            c["clientId"] for c in self.keycloak_admin.get_clients()
        ]

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
            debug(f"Team {team.name} opted out of creating OIDC clients, skipping...")

        # Sync the team leads and service accounts to the Keycloak admins group
        lead_group_name = f"{team.slug}{self.ADMIN_SUFFIX}"
        lead_usernames = self.get_usernames(team.leads)
        # Add the service accounts to the leads only if the OIDC clients are created
        if team.create_oidc_clients:
            lead_usernames = lead_usernames.union(
                self.get_service_account_usernames(team.slug),
            )
        self.sync_group(
            team.name,
            lead_group_name,
            lead_usernames,
            remove_unlisted=team.remove_unlisted,
        )

        # Sync team devs to Keycloak devs group
        member_group_name = f"{team.slug}{self.MEMBER_SUFFIX}"
        members_usernames = self.get_usernames(team.devs)
        self.sync_group(
            team.name,
            member_group_name,
            members_usernames,
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
            error(f"Website slug is not set for team {team_slug}")
            return

        for env in ENVS:
            client_id = f"{team_slug}-{env}"
            if client_id in self.existing_clients:
                debug(f"Keycloak client {client_id} already exists, skipping...")
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
        group_id = group["id"]
        group_name = group["name"]

        # Get the usernames of the current members in the Keycloak group
        members = self.keycloak_admin.get_group_members(group_id)
        current_usernames = {m["username"].lower() for m in members}

        # Calculate new members
        new_members = target_usernames - current_usernames
        debug(f"Found {len(new_members)} new members for the {group_name} group.")

        # Add missing users
        for username in new_members:
            user_id = self.get_user_id_by_username(username)
            if user_id is None:
                continue

            with log_operation(f"add {username} to Keycloak group {group_name}"):
                self.keycloak_admin.group_user_add(user_id, group_id)

        # Remove extra users if the team want to remove unlisted members
        if not remove_unlisted:
            debug(
                f"Team {team_name} opted out of removing unlisted members, skipping..."
            )
            return

        # Calculate unlisted members
        unlisted_members = current_usernames - target_usernames
        debug(
            f"Found {len(unlisted_members)} unlisted members for the "
            f"{group_name} group."
        )

        # Remove unlisted members
        for member in unlisted_members:
            log_message = f"remove {username} from Keycloak group {group_name}"
            with log_operation(log_message):
                self.keycloak_admin.group_user_remove(member["id"], group_id)

    def get_or_create_group(self, group_path: str) -> dict:
        try:
            return self.keycloak_admin.get_group_by_path(group_path)
        except Exception:
            with log_operation(f"create Keycloak group {group_path}"):
                self.keycloak_admin.create_group(payload={"name": group_path})
                return self.keycloak_admin.get_group_by_path(group_path)

    def get_user_id_by_username(self, username: str) -> str | None:
        users = self.keycloak_admin.get_users(
            query={"username": username, "exact": True},
        )

        if not users:
            error(f"User {username} not found in Keycloak!\n")
            return None

        # Used `exact` = True, so this technically should never happen
        if len(users) > 1:
            error(f"Multiple users found for {username}: {users}!\n")
            return None

        return users[0]["id"]
