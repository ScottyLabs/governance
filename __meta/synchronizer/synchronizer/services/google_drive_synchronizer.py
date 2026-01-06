import os
from typing import ClassVar, Literal

from google.oauth2 import service_account
from googleapiclient.discovery import build

from synchronizer.logger import log_operation, print_section
from synchronizer.models import Contributor, Team

from .abstract_synchronizer import AbstractSynchronizer


class GoogleDriveSynchronizer(AbstractSynchronizer):
    drive_role = Literal["writer", "fileOrganizer"]
    drive_role_to_role_name: ClassVar[dict[drive_role, str]] = {
        "writer": "contributor",
        "fileOrganizer": "content manager",
    }

    GOOGLE_AUTH_URI = "https://accounts.google.com/o/oauth2/auth"
    GOOGLE_TOKEN_URI = "https://oauth2.googleapis.com/token"  # noqa: S105
    GOOGLE_UNIVERSE_DOMAIN = "googleapis.com"

    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        super().__init__(contributors, teams)

        # Validate environment variables
        for env_var in [
            "GOOGLE_PROJECT_ID",
            "GOOGLE_PRIVATE_KEY_ID",
            "GOOGLE_PRIVATE_KEY",
            "GOOGLE_CLIENT_EMAIL",
            "GOOGLE_CLIENT_ID",
            "GOOGLE_AUTH_PROVIDER_X509_CERT_URL",
            "GOOGLE_CLIENT_X509_CERT_URL",
            "SCOTTYLABS_GOOGLE_DRIVE_ID",
        ]:
            if env_var not in os.environ:
                msg = f"Environment variable {env_var} is not set"
                self.logger.critical(msg)
                raise RuntimeError(msg)

        # Initialize the Google Drive client
        self.google_drive_id = os.getenv("SCOTTYLABS_GOOGLE_DRIVE_ID")
        creds = service_account.Credentials.from_service_account_info(
            info={
                "type": "service_account",
                "project_id": os.getenv("GOOGLE_PROJECT_ID"),
                "private_key_id": os.getenv("GOOGLE_PRIVATE_KEY_ID"),
                "private_key": os.getenv("GOOGLE_PRIVATE_KEY"),
                "client_email": os.getenv("GOOGLE_CLIENT_EMAIL"),
                "client_id": os.getenv("GOOGLE_CLIENT_ID"),
                "auth_uri": self.GOOGLE_AUTH_URI,
                "token_uri": self.GOOGLE_TOKEN_URI,
                "auth_provider_x509_cert_url": os.getenv(
                    "GOOGLE_AUTH_PROVIDER_X509_CERT_URL"
                ),
                "client_x509_cert_url": os.getenv("GOOGLE_CLIENT_X509_CERT_URL"),
                "universe_domain": self.GOOGLE_UNIVERSE_DOMAIN,
            },
        )

        self.service = build("drive", "v3", credentials=creds)

        # Validate the credentials by making a small request
        try:
            self.service.about().get(fields="user").execute()
        except Exception as e:
            msg = "Failed to initialize Google Drive client"
            self.logger.critical(msg)
            raise RuntimeError(msg) from e

    def sync(self) -> None:
        print_section("Google Drive")

        permissions = self.get_all_permissions(self.service)

        self.add_permissions(
            self.get_new_contributor_email_addresses(permissions), "writer"
        )

        self.add_permissions(
            self.get_new_maintainer_email_addresses(permissions), "fileOrganizer"
        )

    def get_all_permissions(self, service: build) -> list[dict]:
        """Return a email to role mapping for the ScottyLabs Google Drive."""
        permissions = {}
        page_token = None

        while True:
            response = (
                service.permissions()
                .list(
                    fileId=self.google_drive_id,
                    fields="nextPageToken, permissions(emailAddress,role)",
                    supportsAllDrives=True,
                    pageToken=page_token,
                )
                .execute()
            )

            for permission in response.get("permissions", []):
                email_address = permission["emailAddress"]
                role = permission["role"]
                permissions[email_address] = role

            page_token = response.get("nextPageToken")
            if not page_token:
                break

        return permissions

    def get_new_contributor_email_addresses(
        self, permissions: dict[str, str]
    ) -> list[str]:
        new_email_addresses = []
        for contributor in self.contributors.values():
            if contributor.andrew_id is None:
                continue

            email_address = contributor.andrew_id + "@andrew.cmu.edu"
            if email_address not in permissions:
                new_email_addresses.append(email_address)

        return new_email_addresses

    def get_new_maintainer_email_addresses(
        self, permissions: dict[str, str]
    ) -> list[str]:
        new_email_addresses = []
        for team in self.teams.values():
            for maintainer_id in team.maintainers:
                maintainer = self.contributors[maintainer_id]
                if maintainer.andrew_id is None:
                    continue

                email_address = maintainer.andrew_id + "@andrew.cmu.edu"

                # Permission of a maintainer needs to be at least File Organizer
                # organizer role maps to Manager and has more permissions
                if email_address not in permissions or (
                    permissions[email_address] != "fileOrganizer"
                    and permissions[email_address] != "organizer"
                ):
                    new_email_addresses.append(email_address)

        return new_email_addresses

    def add_permissions(self, email_addresses: list[str], role: drive_role) -> None:
        role_name = self.drive_role_to_role_name[role]

        # Log messages
        if len(email_addresses) == 0:
            self.logger.debug("No new %s to add to Google Drive\n", role_name)
            return

        self.logger.info(
            "Adding %d new %s to Google Drive\n", len(email_addresses), role_name
        )

        # Add permissions
        for email_address in email_addresses:
            with log_operation(
                f"add/update {email_address} as a ScottyLabs Google Drive {role_name}"
            ):
                self.service.permissions().create(
                    fileId=self.google_drive_id,
                    body={"type": "user", "role": role, "emailAddress": email_address},
                    supportsAllDrives=True,
                ).execute()
