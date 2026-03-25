use std::collections::HashSet;

use governance_schema::team::{GroupFields, TeamFile};

use crate::error::ValidationError;
use crate::loader::GovernanceData;

pub fn validate(data: &GovernanceData) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    validate_forges(data, &mut errors);
    validate_team_slugs(data, &mut errors);
    validate_repo_names(data, &mut errors);
    validate_groups(data, &mut errors);
    validate_forge_refs(data, &mut errors);

    errors
}

fn validate_forges(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let default_count = data
        .org
        .org
        .forges
        .values()
        .filter(|f| f.default)
        .count();

    if default_count == 0 {
        errors.push(ValidationError::NoDefaultForge);
    } else if default_count > 1 {
        errors.push(ValidationError::MultipleDefaultForges);
    }
}

fn validate_team_slugs(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let mut seen = HashSet::new();
    for team in &data.teams {
        if !seen.insert(&team.team.group.slug) {
            errors.push(ValidationError::DuplicateTeamSlug(
                team.team.group.slug.clone(),
            ));
        }
    }
}

fn validate_repo_names(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let mut seen = HashSet::new();
    for team in &data.teams {
        for repo in all_repos(team) {
            if !seen.insert(&repo.name) {
                errors.push(ValidationError::DuplicateRepoName(repo.name.clone()));
            }
        }
    }
}

fn validate_groups(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    for team in &data.teams {
        validate_leads_not_members(&team.team.group, errors);
        for project in &team.team.projects {
            validate_leads_not_members(&project.group, errors);
        }
    }
}

fn validate_leads_not_members(group: &GroupFields, errors: &mut Vec<ValidationError>) {
    let leads: HashSet<&str> = group.leads.iter().map(|s| s.as_str()).collect();
    for member in &group.members {
        if leads.contains(member.as_str()) {
            errors.push(ValidationError::LeadAlsoMember {
                team: group.slug.clone(),
                lead: member.clone(),
            });
        }
    }
}

fn validate_forge_refs(data: &GovernanceData, errors: &mut Vec<ValidationError>) {
    let forge_names: HashSet<&str> = data.org.org.forges.keys().map(|s| s.as_str()).collect();
    for team in &data.teams {
        let slug = &team.team.group.slug;
        for repo in all_repos(team) {
            if let Some(forge) = &repo.forge {
                if !forge_names.contains(forge.as_str()) {
                    errors.push(ValidationError::UnknownForge {
                        team: slug.clone(),
                        forge: forge.clone(),
                    });
                }
            }
        }
    }
}

fn all_repos(team: &TeamFile) -> Vec<&governance_schema::team::Repo> {
    let mut repos: Vec<_> = team.team.group.repos.iter().collect();
    for project in &team.team.projects {
        repos.extend(project.group.repos.iter());
    }
    repos
}
