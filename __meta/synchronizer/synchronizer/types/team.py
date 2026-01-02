# Automatically generated file from a JSON schema


from typing import Required, TypedDict


# | allOf:
# |   - $comment: Require 'website-slug' when 'create-oidc-clients' is missing or true.
# |     if:
# |       anyOf:
# |       - not:
# |           required:
# |           - create-oidc-clients
# |       - properties:
# |           create-oidc-clients:
# |             const: true
# |     then:
# |       required:
# |       - website-slug
Team = TypedDict(
    "Team",
    {
        # | Used for internal references (e.g. OIDC client ID).
        # |
        # | Required property
        "slug": Required[str],
        # | Used for any public-facing or display contexts (e.g. the GitHub team name).
        # |
        # | Required property
        "name": Required[str],
        # | Used in website URLs (e.g. <website-slug>.scottylabs.org).
        "website-slug": str,
        # | The github usernames of the tech leads of the team.
        # |
        # | minItems: 1
        # |
        # | Required property
        "leads": Required[list[str]],
        # | The github usernames of the developers of the team.
        # |
        # | Required property
        "devs": Required[list[str]],
        # | The github usernames of the applicants of the team.
        "applicants": list[str],
        # | External individuals who need admin access to the app (e.g. orientation staffs for O-Quest) but are not involved in development.
        "ext-admins": list[str],
        # | The github repositories of the team, in the format of 'ScottyLabs/<repo-name>'.
        # |
        # | minItems: 1
        # |
        # | Required property
        "repos": Required[list[str]],
        # | The Slack channel IDs of the team.
        # |
        # | Required property
        "slack-channel-ids": Required[list["_TeamSlackChannelIdsItem"]],
        # | Whether to remove unlisted members from the team.
        # |
        # | default: True
        "remove-unlisted": bool,
        # | Whether to create OIDC clients for the team.
        # |
        # | default: True
        "create-oidc-clients": bool,
        # | The secrets folder layout when populating secrets to HashiCorp Vault.
        # |
        # | oneOf:
        # |   - const: single
        # |     description: 'single: Populate secrets to a single folder.'
        # |   - const: multi
        # |     description: 'multi: Populate secrets to multiple folders, one for each app (e.g.
        # |       one for web and one for server).'
        # |   - const: none
        # |     description: 'none: No secrets population.'
        # | default: multi
        "secrets-population-layout": str,
    },
    total=False,
)


_TEAM_CREATE_OIDC_CLIENTS_DEFAULT = True
r""" Default value of the field path 'Team create-oidc-clients' """


_TEAM_REMOVE_UNLISTED_DEFAULT = True
r""" Default value of the field path 'Team remove-unlisted' """


_TEAM_SECRETS_POPULATION_LAYOUT_DEFAULT = "multi"
r""" Default value of the field path 'Team secrets-population-layout' """


_TeamSlackChannelIdsItem = str
r""" pattern: ^[CG][A-Z0-9]+$ """
