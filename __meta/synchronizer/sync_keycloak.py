from keycloak import KeycloakAdmin
import os
from styler import Styler, error


class KeycloakManager:
    LEAD_SUFFIX = "-leads"
    ADMIN_SUFFIX = "-admins"
    MEMBER_SUFFIX = "-devs"

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

    def sync(self):
        with Styler("Keycloak"):
            for team_slug, team in self.teams.items():
                print(f"\nSyncing team {team_slug}...")

                # Sync the team leads to the Keycloak admins group
                lead_group_name = f"{team_slug}{self.LEAD_SUFFIX}"
                lead_andrew_ids = self.get_andrew_ids(team["leads"])
                self.sync_group(lead_group_name, lead_andrew_ids)

                # Sync team devs to Keycloak devs group
                member_group_name = f"{team_slug}{self.MEMBER_SUFFIX}"
                members_andrew_ids = self.get_andrew_ids(team["devs"])
                self.sync_group(member_group_name, members_andrew_ids)

                # Sync team admins to Keycloak admins group
                admin_group_name = f"{team_slug}{self.ADMIN_SUFFIX}"
                admins_andrew_ids = set(team["admins"])
                self.sync_group(admin_group_name, admins_andrew_ids)

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

        # --- Add missing users ---
        for andrew_id in target_andrew_ids:
            if andrew_id not in current_andrew_ids:
                user_id = self.get_user_id_by_andrew_id(andrew_id)
                if user_id:
                    print(f"Adding {andrew_id} to Keycloak {group_name}...")
                    self.keycloak_admin.group_user_add(user_id, group_id)

        # --- Remove extra users ---
        for member in members:
            andrew_id = member["username"]
            if andrew_id not in target_andrew_ids:
                print(f"Removing {andrew_id} from Keycloak {group_name}...")
                self.keycloak_admin.group_user_remove(member["id"], group_id)

    def get_or_create_group(self, group_path: str):
        group = self.keycloak_admin.get_group_by_path(group_path)
        if "error" in group:
            print(f"Group {group_path} not found in Keycloak, creating...")
            group = self.keycloak_admin.create_group(payload={"name": group_path})
            group = self.keycloak_admin.get_group_by_path(group_path)
        return group

    # Get the user ID by email
    def get_user_id_by_andrew_id(self, andrew_id: str):
        users = self.keycloak_admin.get_users(query={"username": andrew_id})

        if not users:
            error(f"User {andrew_id} not found in Keycloak!", print_traceback=False)
            return False

        if len(users) > 1:
            error(
                f"Multiple users found for {andrew_id}: {users}!", print_traceback=False
            )
            return False

        return users[0]["id"]
