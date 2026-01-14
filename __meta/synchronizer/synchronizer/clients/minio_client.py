import os
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

    def get_service_accounts(self) -> list[dict[str, Any]]:
        """Get the list of service accounts."""
        response = requests.get(
            f"{self.ENDPOINT}/api/v1/service-accounts",
            headers={"Cookie": self.token},
            timeout=self.TIMEOUT,
        )
        response.raise_for_status()
        return response.json()  # type: ignore[no-any-return]


@lru_cache(maxsize=1)
def get_minio_client() -> MinioClient:
    """Get the Minio client. Cache the result for reuse across the app."""
    return MinioClient()


def main() -> None:
    client = get_minio_client()
    service_accounts = client.get_service_accounts()
    for service_account in service_accounts:
        print(service_account)
