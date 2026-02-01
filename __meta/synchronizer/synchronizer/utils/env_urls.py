from typing import Literal

# List of environments
ENVS_LITERAL = Literal["local", "dev", "staging", "prod"]
ENVS: list[ENVS_LITERAL] = ["local", "dev", "staging", "prod"]


def get_server_url(website_slug: str, env: ENVS_LITERAL) -> str:
    match env:
        case "local":
            return "http://localhost"
        case "dev":
            return f"https://api.{website_slug}.slabs-dev.org"
        case "staging":
            return f"https://api.{website_slug}.slabs-staging.org"
        case "prod":
            return f"https://api.{website_slug}.scottylabs.org"


def get_frontend_url(website_slug: str, env: ENVS_LITERAL) -> str:
    match env:
        case "local":
            return "http://localhost:3000"
        case "dev":
            return f"https://{website_slug}.slabs-dev.org"
        case "staging":
            return f"https://{website_slug}.slabs-staging.org"
        case "prod":
            return f"https://{website_slug}.scottylabs.org"


def get_allowed_origins_regex(team_slug: str, env: ENVS_LITERAL) -> str:
    # Allow any https prefix
    https_origin_prefix = r"^https://([a-z0-9-]+\.)*"

    # Populate the allowed origins regex
    match env:
        case "local":
            # Allow all origins for local development
            return r"^https?://localhost:\d{4}$"
        case "dev":
            # Allow all ScottyLabs dev subdomains and any vercel preview domains
            # (https://<team-slug>-<random 9 characters>-scottylabs.vercel.app)
            # for dev development
            return (
                rf"{https_origin_prefix}slabs-dev\.org$,"
                rf"^https://{team_slug}-[0-9a-z]{{9}}-scottylabs\.vercel\.app$"
            )
        case "staging":
            # Allow all ScottyLabs staging subdomains for staging development
            return rf"{https_origin_prefix}slabs-staging\.org$"
        case "prod":
            # Allow all ScottyLabs production subdomains for production
            return rf"{https_origin_prefix}scottylabs\.org$"
