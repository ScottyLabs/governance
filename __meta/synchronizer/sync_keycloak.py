import os

from keycloak import KeycloakAdmin
from utils import ENVS, log_operation, log_team_sync, print_section, warn


class KeycloakManager:
    ADMIN_SUFFIX = "-admins"
    EXTERNAL_ADMIN_SUFFIX = "-ext-admins"
    MEMBER_SUFFIX = "-devs"
    APPLICANT_SUFFIX = "-applicants"

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

        # Create the client if it does not exist
        for env in ENVS:
            client_id = f"{team_slug}-{env}"
            if client_id not in self.existing_clients:
                self.create_client(team_slug, team["website-slug"], env)

        # Sync the team leads to the Keycloak admins group
        lead_group_name = f"{team_slug}{self.ADMIN_SUFFIX}"
        lead_andrew_ids = self.get_andrew_ids(team["leads"])
        self.sync_group(lead_group_name, lead_andrew_ids)

        # Sync team devs to Keycloak devs group
        member_group_name = f"{team_slug}{self.MEMBER_SUFFIX}"
        members_andrew_ids = self.get_andrew_ids(team["devs"])
        self.sync_group(member_group_name, members_andrew_ids)

        # Sync team admins to Keycloak admins group
        admin_group_name = f"{team_slug}{self.EXTERNAL_ADMIN_SUFFIX}"
        admins_andrew_ids = set(team["ext-admins"])
        self.sync_group(admin_group_name, admins_andrew_ids)

        if "applicants" in team:
            applicant_group_name = f"{team_slug}{self.APPLICANT_SUFFIX}"
            applicants_andrew_ids = self.get_andrew_ids(team["applicants"])
            self.sync_group(applicant_group_name, applicants_andrew_ids)

    def create_client(self, team_slug: str, website_slug: str, env: str):
        client_id = f"{team_slug}-{env}"
        with log_operation(f"create Keycloak client {client_id}"):
            # Generate the URIs for the client
            rootUrl = None
            match env:
                case "dev":
                    rootUrl = f"https://{website_slug}.slabs-dev.org"
                    serverUrl = f"https://api.{website_slug}.slabs-dev.org"
                case "staging":
                    rootUrl = f"https://{website_slug}.slabs-staging.org"
                    serverUrl = f"https://api.{website_slug}.slabs-staging.org"
                case "prod":
                    rootUrl = f"https://{website_slug}.scottylabs.org"
                    serverUrl = f"https://api.{website_slug}.scottylabs.org"

            if env == "local":
                redirectUris = ["http://localhost/auth/callback"]
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
                    # Add the groups claim to the token
                    "protocolMappers": [
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
                        }
                    ],
                }
            )

    def get_andrew_ids(self, members: list[str]):
        andrew_ids = set()
        for member in members:
            # Validation check guarantees that members will always be a contributor
            if "andrew-id" in self.contributors[member]:
                andrew_ids.add(self.contributors[member]["andrew-id"])
        return andrew_ids

    def sync_group(self, group_path: str, target_andrew_ids: set[str]):
        group = self.get_or_create_group(group_path)
        group_id = group["id"]
        group_name = group["name"]

        # Get the andrew ids of the current members in the Keycloak group
        members = self.keycloak_admin.get_group_members(group_id)
        current_andrew_ids = {m["username"].lower() for m in members}

        # Add missing users
        for andrew_id in target_andrew_ids:
            if andrew_id not in current_andrew_ids:
                user_id = self.get_user_id_by_andrew_id(andrew_id)
                if not user_id:
                    continue

                with log_operation(f"add {andrew_id} to Keycloak group {group_name}"):
                    self.keycloak_admin.group_user_add(user_id, group_id)

        # Remove extra users
        for member in members:
            andrew_id = member["username"]
            if andrew_id in target_andrew_ids:
                continue

            with log_operation(f"remove {andrew_id} from Keycloak group {group_name}"):
                self.keycloak_admin.group_user_remove(member["id"], group_id)

    def get_or_create_group(self, group_path: str):
        try:
            return self.keycloak_admin.get_group_by_path(group_path)
        except Exception:
            with log_operation(f"create Keycloak group {group_path}"):
                self.keycloak_admin.create_group(payload={"name": group_path})
                return self.keycloak_admin.get_group_by_path(group_path)

    # Get the user ID by email
    def get_user_id_by_andrew_id(self, andrew_id: str):
        users = self.keycloak_admin.get_users(query={"username": andrew_id})

        if not users:
            warn(f"User {andrew_id} not found in Keycloak!")
            return False

        if len(users) > 1:
            warn(f"Multiple users found for {andrew_id}: {users}!")
            return False

        return users[0]["id"]
