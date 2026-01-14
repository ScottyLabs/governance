import json
import os
import secrets
import string
from functools import lru_cache
from typing import Any

import dotenv
import requests

from synchronizer.logger import get_app_logger

dotenv.load_dotenv()


class MinioClient:
    ENDPOINT = "https://minio.scottylabs.org"
    TIMEOUT = 10

    def __init__(self) -> None:
        """Initialize the Minio client by logging in and getting the token."""
        self.logger = get_app_logger()

        response = requests.post(
            f"{self.ENDPOINT}/api/v1/login",
            json={
                "accessKey": os.getenv("MINIO_ACCESS_KEY"),
                "secretKey": os.getenv("MINIO_SECRET_KEY"),
            },
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()
        self.token = response.headers["Set-Cookie"]

    def get_buckets(self) -> list[dict[str, Any]]:
        """Get the list of buckets."""
        response = requests.get(
            f"{self.ENDPOINT}/api/v1/buckets",
            headers={"Cookie": self.token},
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()
        return response.json()["buckets"]  # type: ignore[no-any-return]

    def create_bucket(self, bucket_name: str) -> None:
        """Create a bucket."""
        response = requests.post(
            f"{self.ENDPOINT}/api/v1/buckets",
            json={
                "name": bucket_name,
                "versioning": {"enabled": False},  # no versioning
                "quota": {
                    "enabled": True,
                    "quota_type": "hard",  # hard quota
                    "amount": 1024 * 1024 * 1024,  # 1GB
                },
            },
            headers={"Cookie": self.token},
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()

    def get_service_accounts(self) -> list[dict[str, Any]]:
        """Get the list of service accounts."""
        response = requests.get(
            f"{self.ENDPOINT}/api/v1/service-accounts",
            headers={"Cookie": self.token},
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()
        return response.json()  # type: ignore[no-any-return]

    def create_service_account(
        self, service_account_name: str, description: str, policy: dict[str, Any]
    ) -> None:
        """Create a service account."""
        alphabet = string.ascii_letters + string.digits
        access_key = "".join(
            secrets.choice(alphabet) for _ in range(20)
        )  # 20 random characters
        secret_key = "".join(
            secrets.choice(alphabet) for _ in range(40)
        )  # 40 random characters

        response = requests.post(
            f"{self.ENDPOINT}/api/v1/service-account-credentials",
            json={
                "name": service_account_name,
                "access_key": access_key,
                "secret_key": secret_key,
                "description": description,
                "policy": json.dumps(policy),
            },
            headers={"Cookie": self.token},
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()


@lru_cache(maxsize=1)
def get_minio_client() -> MinioClient:
    """Get the Minio client. Cache the result for reuse across the app."""
    return MinioClient()
