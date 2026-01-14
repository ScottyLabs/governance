from collections.abc import Callable
from typing import Any, override

from synchronizer.clients import get_minio_client
from synchronizer.logger import log_operation, log_team_sync, print_section
from synchronizer.models import Contributor, Team

from .abstract_synchronizer import AbstractSynchronizer


class MinioSynchronizer(AbstractSynchronizer):
    def __init__(
        self, contributors: dict[str, Contributor], teams: dict[str, Team]
    ) -> None:
        super().__init__(contributors, teams)
        self.minio_client = get_minio_client()

        print_section("MinIO")

        try:
            self.logger.info("Getting existing MinIO buckets and service accounts...")
            self.existing_buckets = {
                bucket["name"] for bucket in self.minio_client.get_buckets()
            }
            self.existing_service_accounts = {
                sa["name"]
                for sa in self.minio_client.get_service_accounts()
                if sa.get("name") is not None  # some service accounts don't have a name
            }
            self.logger.success(
                "Successfully got existing MinIO buckets and service accounts.\n"
            )
        except Exception:
            self.logger.exception(
                "Failed to get existing MinIO buckets or service accounts"
            )
            raise

    @override
    def sync(self) -> None:
        for team in self.teams.values():
            self.sync_team(team)

    @log_team_sync()
    def sync_team(self, team: Team) -> None:
        if not team.sync_minio:
            self.logger.debug(
                "Team %s opted out of synchronizing with MinIO, skipping...\n",
                team.name,
            )
            return

        self.sync_bucket(team.slug)
        self.sync_service_account(team, "Read-Only", self.get_read_only_policy)
        self.sync_service_account(team, "Admin", self.get_admin_policy)

    def sync_bucket(self, team_slug: str) -> None:
        if team_slug in self.existing_buckets:
            self.logger.debug(
                "MinIO bucket %s already exists, skipping...\n",
                team_slug,
            )
            return

        with log_operation(f"create MinIO bucket {team_slug}"):
            self.minio_client.create_bucket(team_slug)

    def sync_service_account(
        self,
        team: Team,
        permission: str,
        get_policy: Callable[[str], dict[str, Any]],
    ) -> None:
        service_account_name = f"{team.name} {permission}"

        if service_account_name in self.existing_service_accounts:
            self.logger.debug(
                "MinIO service account %s already exists, skipping...\n",
                service_account_name,
            )
            return

        with log_operation(f"create MinIO service account {service_account_name}"):
            access_key, secret_key = self.minio_client.create_service_account(
                service_account_name,
                f"{permission} access to {team.slug} bucket",
                get_policy(team.slug),
            )

            if permission == "Read-Only":
                team.minio_readonly_access_key = access_key
                team.minio_readonly_secret_key = secret_key
            elif permission == "Admin":
                team.minio_admin_access_key = access_key
                team.minio_admin_secret_key = secret_key

    def get_read_only_policy(self, team_slug: str) -> dict[str, Any]:
        return {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": ["s3:GetBucketLocation", "s3:ListBucket"],
                    "Resource": [f"arn:aws:s3:::{team_slug}"],
                },
                {
                    "Effect": "Allow",
                    "Action": ["s3:GetObject"],
                    "Resource": [f"arn:aws:s3:::{team_slug}/*"],
                },
            ],
        }

    def get_admin_policy(self, team_slug: str) -> dict[str, Any]:
        return {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": ["s3:*"],
                    "Resource": [f"arn:aws:s3:::{team_slug}/*"],
                },
            ],
        }
