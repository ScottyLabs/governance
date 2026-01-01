import os

from keycloak import KeycloakAdmin
from utils import (
    ENVS,
    get_dev_server_url,
    get_local_server_url,
    get_prod_server_url,
    get_staging_server_url,
    log_operation,
    log_team_sync,
    print_section,
    warn,
)


class KeycloakManager:
    ADMIN_SUFFIX = "-admins"
    EXTERNAL_ADMIN_SUFFIX = "-ext-admins"
    MEMBER_SUFFIX = "-devs"
    APPLICANT_SUFFIX = "-applicants"
    SERVICE_ACCOUNT_PREFIX = "service-account-"

    def __init__(self, contributors, teams):
        self.contributors = contributors
        self.teams = teams
        self.keycloak_admin = KeycloakAdmin(
            server_url=os.getenv("KEYCLOAK_SERVER_URL"),
            username=os.getenv("KEYCLOAK_USERNAME"),
            password=os.getenv("KEYCLOAK_PASSWORD"),
            realm_name=os.getenv("KEYCLOAK_REALM"),
            client_id=os.getenv("KEYCLOAK_CLIENT_ID"),
            user_realm_name=os.getenv("KEYCLOAK_USER_REALM"),
            verify=True,
        )
        self.existing_clients = [
            c["clientId"] for c in self.keycloak_admin.get_clients()
        ]

    def sync(self):
        print_section("Keycloak")
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team):
        team_slug = team["slug"]
        remove_unlisted = team.get("remove-unlisted", True)

        # Create the OIDC clients for the team if the team wants
        create_oidc_clients = team.get("create-oidc-clients", True)
        if create_oidc_clients:
            self.create_clients(team_slug, team)

        # Sync the team leads and service accounts to the Keycloak admins group
        lead_group_name = f"{team_slug}{self.ADMIN_SUFFIX}"
        lead_usernames = self.get_usernames(team["leads"])
        # Add the service accounts to the leads only if the OIDC clients are created
        if create_oidc_clients:
            lead_usernames = lead_usernames.union(
                self.get_service_account_usernames(team_slug)
            )
        self.sync_group(lead_group_name, lead_usernames, remove_unlisted)

        # Sync team devs to Keycloak devs group
        member_group_name = f"{team_slug}{self.MEMBER_SUFFIX}"
        members_usernames = self.get_usernames(team["devs"])
        self.sync_group(member_group_name, members_usernames, remove_unlisted)

        # Sync team admins to Keycloak admins group
        if "ext-admins" in team:
            admin_group_name = f"{team_slug}{self.EXTERNAL_ADMIN_SUFFIX}"
            admins_usernames = set(team["ext-admins"])
            self.sync_group(admin_group_name, admins_usernames, remove_unlisted)

        if "applicants" in team:
            applicant_group_name = f"{team_slug}{self.APPLICANT_SUFFIX}"
            applicants_usernames = self.get_usernames(team["applicants"])
            self.sync_group(applicant_group_name, applicants_usernames, remove_unlisted)

    def create_clients(self, team_slug, team):
        for env in ENVS:
            client_id = f"{team_slug}-{env}"
            if client_id not in self.existing_clients:
                with log_operation(f"create Keycloak client {client_id}"):
                    self.create_client(client_id, team["website-slug"], env)

    def create_client(self, client_id: str, website_slug: str, env: str):
        # Generate the URIs for the client
        rootUrl = None
        match env:
            case "dev":
                rootUrl = f"https://{website_slug}.slabs-dev.org"
                serverUrl = get_dev_server_url(website_slug)
            case "staging":
                rootUrl = f"https://{website_slug}.slabs-staging.org"
                serverUrl = get_staging_server_url(website_slug)
            case "prod":
                rootUrl = f"https://{website_slug}.scottylabs.org"
                serverUrl = get_prod_server_url(website_slug)

        if env == "local":
            redirectUris = [f"{get_local_server_url()}/auth/callback"]
            post_logout_redirect_uris = "http://localhost:3000/*"
        else:
            redirectUris = [f"{serverUrl}/auth/callback"]
            # Permit any post-logout redirect URI with the same origin
            post_logout_redirect_uris = "/*"

        # Create the client
        self.keycloak_admin.create_client(
            payload={
                "clientId": client_id,
                "rootUrl": rootUrl,
                "redirectUris": redirectUris,
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
            }
        )

    def get_usernames(self, members: list[str]):
        usernames = set()
        for member in members:
            # Validation check guarantees that members will always be a contributor
            if "andrew-id" in self.contributors[member]:
                # The andrew id is the username in Keycloak
                usernames.add(self.contributors[member]["andrew-id"])
        return usernames

    def get_service_account_usernames(self, team_slug: str):
        # The service account usernames should all follow the pattern
        # "service-account-<team-slug>-<env>" (e.g. "service-account-graph-local")
        return set([f"{self.SERVICE_ACCOUNT_PREFIX}{team_slug}-{env}" for env in ENVS])

    def sync_group(self, group_path: str, target_usernames, remove_unlisted):
        group = self.get_or_create_group(group_path)
        group_id = group["id"]
        group_name = group["name"]

        # Get the usernames of the current members in the Keycloak group
        members = self.keycloak_admin.get_group_members(group_id)
        current_usernames = {m["username"].lower() for m in members}

        # Add missing users
        for username in target_usernames:
            if username not in current_usernames:
                user_id = self.get_user_id_by_username(username)
                if not user_id:
                    continue

                with log_operation(f"add {username} to Keycloak group {group_name}"):
                    self.keycloak_admin.group_user_add(user_id, group_id)

        # Remove extra users if the team want to remove unlisted members
        if remove_unlisted:
            for member in members:
                username = member["username"]
                if username in target_usernames:
                    continue

                log_message = f"remove {username} from Keycloak group {group_name}"
                with log_operation(log_message):
                    self.keycloak_admin.group_user_remove(member["id"], group_id)

    def get_or_create_group(self, group_path: str):
        try:
            return self.keycloak_admin.get_group_by_path(group_path)
        except Exception:
            with log_operation(f"create Keycloak group {group_path}"):
                self.keycloak_admin.create_group(payload={"name": group_path})
                return self.keycloak_admin.get_group_by_path(group_path)

    def get_user_id_by_username(self, username: str):
        users = self.keycloak_admin.get_users(
            query={"username": username, "exact": True}
        )

        if not users:
            warn(f"User {username} not found in Keycloak!\n")
            return False

        # Used `exact` = True, so this technically should never happen
        if len(users) > 1:
            warn(f"Multiple users found for {username}: {users}!\n")
            return False

        return users[0]["id"]
