from typing import Literal

# List of environments
ENVS_LITERAL = Literal["local", "dev", "staging", "prod"]
ENVS: list[ENVS_LITERAL] = ["local", "dev", "staging", "prod"]


def get_server_url(website_slug: str, env: ENVS_LITERAL) -> str:
    match env:
        case "local":
            return get_local_server_url()
        case "dev":
            return get_dev_server_url(website_slug)
        case "staging":
            return get_staging_server_url(website_slug)
        case "prod":
            return get_prod_server_url(website_slug)


def get_local_server_url() -> str:
    return "http://localhost"


def get_dev_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.slabs-dev.org"


def get_staging_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.slabs-staging.org"


def get_prod_server_url(website_slug: str) -> str:
    return f"https://api.{website_slug}.scottylabs.org"


def get_frontend_url(website_slug: str, env: ENVS_LITERAL) -> str:
    match env:
        case "local":
            return get_local_frontend_url()
        case "dev":
            return get_dev_frontend_url(website_slug)
        case "staging":
            return get_staging_frontend_url(website_slug)
        case "prod":
            return get_prod_frontend_url(website_slug)


def get_local_frontend_url() -> str:
    return "http://localhost:3000"


def get_dev_frontend_url(website_slug: str) -> str:
    return f"https://{website_slug}.slabs-dev.org"


def get_staging_frontend_url(website_slug: str) -> str:
    return f"https://{website_slug}.slabs-staging.org"


def get_prod_frontend_url(website_slug: str) -> str:
    return f"https://{website_slug}.scottylabs.org"
