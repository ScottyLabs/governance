# Security Policy

This repository is the source of truth for ScottyLabs Tech Committee governance. It manages teams, repository access, identity groups, secrets, deployment hooks, and project integrations through OpenTofu and Atlantis.

Security issues in this repository can affect more than one project. Report vulnerabilities privately and avoid posting sensitive details in public issues, pull requests, Discord, Slack, or other public channels.

## Scope

This policy applies to this repository and to the organization resources managed from it, including:

- ScottyLabs team and repository membership
- Codeberg repositories and GitHub mirrors
- Keycloak groups, roles, and OIDC clients
- OpenBao policies and project secret paths
- Vaultwarden account access
- Kennel deployment hooks
- Sentry, PostHog, CDN, and LiteLLM project credentials
- Discord, Slack, and Matrix channel membership or bridge configuration

Individual ScottyLabs projects may publish their own `SECURITY.md` files. Follow a project-specific policy when it is stricter or more specific. Otherwise, this policy is the default for governance-managed access and infrastructure.

## Trust Model

Governance changes are trusted when they are reviewed, attributable, and applied through declared infrastructure. Membership, generated files, automation output, and third-party platform state are not trustworthy on their own.

### Trusted Roles

- Team leads are trusted to manage membership and project access for teams they lead.
- Infrastructure maintainers are trusted to review organization-wide access, secrets, identity, deployment, and provider configuration.

### Trust Boundaries

- Codeberg pull requests are the review boundary for governance changes.
- OpenTofu plans and Atlantis output are trusted only after review against the source data and intended access model.
- Keycloak groups, OpenBao policies, repository permissions, deployment hooks, and third-party integrations are security boundaries because they can grant access to code, credentials, production services, or user data.

### Non-Authoritative State

- Access after a member leaves a team or role
- Generated files that have not been reviewed
- Third-party platform permissions that conflict with this repository's declared governance state
- Equal risk across development, staging, preview, and production
- Public access to secrets, logs, dashboards, deployment controls, or private operational details

When platform state and this repository disagree, maintainers should investigate the drift and prefer the least-privileged state until the discrepancy is resolved.

## Reporting a Vulnerability

Report security issues privately to the Tech Committee leads or another trusted ScottyLabs infrastructure maintainer. Project leads are listed in `data/teams/<project name>.toml`.

When reporting an issue, include as much of the following as you can:

- The affected repository, service, integration, or account
- A short description of the security impact
- Steps to reproduce or evidence of the issue
- Whether any credentials, personal data, production systems, or student records may be affected
- Any immediate containment you believe is necessary
- Your preferred contact method for follow-up

Do not include real secrets, private keys, session tokens, or other sensitive values in the report unless a maintainer specifically asks for a secure transfer method.

## What to Report

Report issues such as:

- Leaked credentials, tokens, API keys, DSNs, deploy keys, or OAuth client secrets
- Unauthorized access to repositories, teams, channels, dashboards, secrets, or production systems
- Incorrect privilege escalation through team membership, admin groups, OIDC clients, OpenBao policies, or repository permissions
- OIDC redirect URI, client, or service-account misconfigurations
- Insecure deployment hooks, CI/CD behavior, or mirror configuration
- Infrastructure changes that unintentionally expose private services or data
- Vulnerabilities in governance tooling that could modify access, secrets, or deployed infrastructure
- Suspicious account activity involving ScottyLabs-managed systems
- Supply-chain issues that affect build, deploy, or governance automation

General bugs, feature requests, documentation issues, and non-sensitive configuration questions should use the normal issue or pull request process.

## Responsible Disclosure

ScottyLabs asks reporters to:

- Give maintainers a reasonable opportunity to investigate and remediate before public disclosure
- Avoid destructive testing, denial of service, social engineering, spam, or phishing
- Avoid accessing, modifying, deleting, or exfiltrating data beyond what is necessary to demonstrate impact
- Avoid testing against production systems when the same issue can be demonstrated safely in development or staging
- Keep vulnerability details private until maintainers confirm that disclosure is appropriate

We will not pursue action against good-faith security research that follows this policy and avoids harm to ScottyLabs users, members, infrastructure, or data.

## Response Process

After receiving a security report, maintainers should:

1. Acknowledge the report within 72 hours.
1. Triage the issue for severity, affected systems, and required containment.
1. Restrict details to maintainers and team leads who need to know.
1. Revoke, rotate, or disable affected credentials, access, clients, hooks, or integrations when needed.
1. Prepare and review a fix through a private workflow when needed.
1. Coordinate disclosure with the reporter after remediation.
1. Record follow-up work so the same class of issue is less likely to recur.

If a report indicates active compromise, prioritize containment over normal review flow.

## Governance Security Requirements

Changes should preserve least privilege and reviewability.

- Team membership changes must be made by the member themselves or by an authorized team lead.
- Changes that grant or expand privileged access require review from a maintainer or responsible team lead.
- Secrets must not be committed to Git. Store secrets in OpenBao, Vaultwarden, or another approved secret store.
- Generated OpenTofu output should reflect reviewed source data and should not be edited by hand unless the repository explicitly documents that workflow.
- Access should be removed promptly when a member leaves a team, project, or role.
- Production access should be narrower than development access whenever the managed platform supports that distinction.

## Security-Sensitive Changes

Request appropriate review for pull requests that modify:

- Team leads, admins, or repository maintainers
- Codeberg, GitHub, Keycloak, OpenBao, Vaultwarden, or infrastructure ownership
- OpenBao policies, secret paths, or secret-writing resources
- OIDC clients, redirect URIs, service accounts, client scopes, or role mappings
- Deployment hooks, webhooks, deploy keys, mirrors, CI/CD permissions, or Atlantis behavior
- Public URLs, CDN buckets, network exposure, or project provisioning features
- Sentry, PostHog, LiteLLM, or other third-party integrations that create credentials or grant organization access

When in doubt, treat a change as security-sensitive.

## Incident Handling

For incidents, maintainers should:

1. Contain the issue by revoking credentials, removing access, disabling clients or hooks, or pausing affected automation.
1. Assess which systems, users, data, and repositories may be affected.
1. Remediate the root cause in configuration, code, infrastructure, or process.
1. Rotate related credentials and verify that access is correct after the fix.
1. Communicate with affected maintainers, teams, users, or partners.
1. Document lessons learned and create follow-up issues for hardening work.

Incident notes should avoid secrets, private user data, or operational details that would help reproduce the issue.

## Credit

ScottyLabs appreciates responsible security reports. With the reporter's permission, maintainers may credit the reporter after the issue is resolved.
