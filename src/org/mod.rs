//! # Multi-Tenant Organization View
//!
//! Provides a three-level hierarchy: [`Organization`] → [`Team`] → [`Project`],
//! each tracking cumulative spend and budget allocation.
//!
//! ## Usage
//!
//! ```rust
//! use llm_cost_dashboard::org::{Organization, Team, Project};
//!
//! let mut org = Organization::new("Acme Corp", 1000.0);
//! org.add_team(Team::new("Platform", 400.0));
//! org.add_team(Team::new("Product", 600.0));
//!
//! org.record_spend("Platform", "Search API", 12.5);
//! let summary = org.summary();
//! println!("Org total: ${:.2}", summary.total_spent_usd);
//! ```

use std::collections::HashMap;

/// A leaf-level project within a team.
#[derive(Debug, Clone)]
pub struct Project {
    /// Project name.
    pub name: String,
    /// Optional budget allocated to this project (USD).
    pub budget_usd: Option<f64>,
    /// Cumulative spend in USD.
    pub spent_usd: f64,
}

impl Project {
    /// Create a new project with an optional budget.
    pub fn new(name: impl Into<String>, budget_usd: Option<f64>) -> Self {
        Self {
            name: name.into(),
            budget_usd,
            spent_usd: 0.0,
        }
    }

    /// Record `amount_usd` of spend against this project.
    pub fn record_spend(&mut self, amount_usd: f64) {
        self.spent_usd += amount_usd.max(0.0);
    }

    /// Remaining budget for this project, if a budget was set.
    pub fn remaining_usd(&self) -> Option<f64> {
        self.budget_usd.map(|b| b - self.spent_usd)
    }

    /// Whether this project is over its budget (returns `false` when no budget is set).
    pub fn is_over_budget(&self) -> bool {
        self.budget_usd.map(|b| self.spent_usd > b).unwrap_or(false)
    }
}

/// A team within an organization, containing multiple projects.
#[derive(Debug, Clone)]
pub struct Team {
    /// Team name.
    pub name: String,
    /// Budget allocated to this team by the organization (USD).
    pub budget_usd: f64,
    /// Projects within this team, keyed by project name.
    pub projects: HashMap<String, Project>,
}

impl Team {
    /// Create a new team with the given name and budget.
    pub fn new(name: impl Into<String>, budget_usd: f64) -> Self {
        Self {
            name: name.into(),
            budget_usd,
            projects: HashMap::new(),
        }
    }

    /// Add or replace a project within this team.
    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.name.clone(), project);
    }

    /// Record spend for a named project.  Creates the project if it does not exist.
    pub fn record_spend(&mut self, project_name: &str, amount_usd: f64) {
        self.projects
            .entry(project_name.to_owned())
            .or_insert_with(|| Project::new(project_name, None))
            .record_spend(amount_usd);
    }

    /// Total spend across all projects in this team.
    pub fn total_spent_usd(&self) -> f64 {
        self.projects.values().map(|p| p.spent_usd).sum()
    }

    /// Remaining budget for this team.
    pub fn remaining_usd(&self) -> f64 {
        self.budget_usd - self.total_spent_usd()
    }

    /// Whether this team has exceeded its allocated budget.
    pub fn is_over_budget(&self) -> bool {
        self.total_spent_usd() > self.budget_usd
    }

    /// Percentage of team budget consumed (0.0–1.0+).
    pub fn pct_consumed(&self) -> f64 {
        if self.budget_usd <= 0.0 {
            return 1.0;
        }
        self.total_spent_usd() / self.budget_usd
    }
}

/// Summary snapshot for a single project.
#[derive(Debug, Clone)]
pub struct ProjectSummary {
    /// Project name.
    pub name: String,
    /// Amount spent in USD.
    pub spent_usd: f64,
    /// Optional budget in USD.
    pub budget_usd: Option<f64>,
    /// Whether the project is over budget.
    pub over_budget: bool,
}

/// Summary snapshot for a single team.
#[derive(Debug, Clone)]
pub struct TeamSummary {
    /// Team name.
    pub name: String,
    /// Total amount spent in USD.
    pub total_spent_usd: f64,
    /// Budget allocated to this team in USD.
    pub budget_usd: f64,
    /// Whether the team is over its budget.
    pub over_budget: bool,
    /// Per-project summaries, sorted by spend descending.
    pub projects: Vec<ProjectSummary>,
}

/// Summary snapshot for the entire organization.
#[derive(Debug, Clone)]
pub struct OrgSummary {
    /// Organization name.
    pub name: String,
    /// Total spend across all teams in USD.
    pub total_spent_usd: f64,
    /// Total org budget in USD.
    pub total_budget_usd: f64,
    /// Whether any team is over budget.
    pub any_over_budget: bool,
    /// Per-team summaries, sorted by spend descending.
    pub teams: Vec<TeamSummary>,
}

/// Top-level organization that owns teams and holds the master budget.
#[derive(Debug, Clone)]
pub struct Organization {
    /// Organization name.
    pub name: String,
    /// Total organization budget in USD (allocated across teams).
    pub total_budget_usd: f64,
    /// Teams within the organization, keyed by team name.
    pub teams: HashMap<String, Team>,
}

impl Organization {
    /// Create a new organization with the given name and total budget.
    pub fn new(name: impl Into<String>, total_budget_usd: f64) -> Self {
        Self {
            name: name.into(),
            total_budget_usd,
            teams: HashMap::new(),
        }
    }

    /// Add or replace a team within this organization.
    pub fn add_team(&mut self, team: Team) {
        self.teams.insert(team.name.clone(), team);
    }

    /// Record spend for a team/project pair.  Creates missing teams and projects
    /// on demand with zero budgets.
    pub fn record_spend(&mut self, team_name: &str, project_name: &str, amount_usd: f64) {
        self.teams
            .entry(team_name.to_owned())
            .or_insert_with(|| Team::new(team_name, 0.0))
            .record_spend(project_name, amount_usd);
    }

    /// Total spend across all teams in USD.
    pub fn total_spent_usd(&self) -> f64 {
        self.teams.values().map(|t| t.total_spent_usd()).sum()
    }

    /// Remaining organization budget in USD.
    pub fn remaining_usd(&self) -> f64 {
        self.total_budget_usd - self.total_spent_usd()
    }

    /// Percentage of total org budget consumed.
    pub fn pct_consumed(&self) -> f64 {
        if self.total_budget_usd <= 0.0 {
            return 1.0;
        }
        self.total_spent_usd() / self.total_budget_usd
    }

    /// Whether any team within the org is over its allocated budget.
    pub fn any_over_budget(&self) -> bool {
        self.teams.values().any(|t| t.is_over_budget())
    }

    /// Build a complete [`OrgSummary`] snapshot (sorted teams → sorted projects).
    pub fn summary(&self) -> OrgSummary {
        let mut teams: Vec<TeamSummary> = self
            .teams
            .values()
            .map(|t| {
                let mut projects: Vec<ProjectSummary> = t
                    .projects
                    .values()
                    .map(|p| ProjectSummary {
                        name: p.name.clone(),
                        spent_usd: p.spent_usd,
                        budget_usd: p.budget_usd,
                        over_budget: p.is_over_budget(),
                    })
                    .collect();
                projects.sort_by(|a, b| {
                    b.spent_usd
                        .partial_cmp(&a.spent_usd)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                TeamSummary {
                    name: t.name.clone(),
                    total_spent_usd: t.total_spent_usd(),
                    budget_usd: t.budget_usd,
                    over_budget: t.is_over_budget(),
                    projects,
                }
            })
            .collect();
        teams.sort_by(|a, b| {
            b.total_spent_usd
                .partial_cmp(&a.total_spent_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        OrgSummary {
            name: self.name.clone(),
            total_spent_usd: self.total_spent_usd(),
            total_budget_usd: self.total_budget_usd,
            any_over_budget: self.any_over_budget(),
            teams,
        }
    }

    /// Render a tree of the org hierarchy to a `Vec<String>` suitable for a
    /// TUI list widget.
    ///
    /// Format:
    /// ```text
    /// [Org] Acme Corp   $45.23 / $1000.00
    ///   [Team] Platform  $30.00 / $400.00
    ///     [Proj] Search API  $18.00
    ///     [Proj] Indexer     $12.00
    ///   [Team] Product   $15.23 / $600.00
    /// ```
    pub fn tree_lines(&self) -> Vec<String> {
        let summary = self.summary();
        let mut lines = Vec::new();
        lines.push(format!(
            "[Org] {}   ${:.2} / ${:.2}  ({:.0}%)",
            summary.name,
            summary.total_spent_usd,
            summary.total_budget_usd,
            summary.total_spent_usd / summary.total_budget_usd.max(0.001) * 100.0,
        ));
        for team in &summary.teams {
            let over = if team.over_budget { " OVER" } else { "" };
            lines.push(format!(
                "  [Team] {}   ${:.2} / ${:.2}{}",
                team.name, team.total_spent_usd, team.budget_usd, over,
            ));
            for proj in &team.projects {
                let over_p = if proj.over_budget { " OVER" } else { "" };
                let budget_str = proj
                    .budget_usd
                    .map(|b| format!(" / ${b:.2}"))
                    .unwrap_or_default();
                lines.push(format!(
                    "    [Proj] {}   ${:.2}{}{over_p}",
                    proj.name, proj.spent_usd, budget_str,
                ));
            }
        }
        lines
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_project_spend_accumulates() {
        let mut p = Project::new("api", Some(100.0));
        p.record_spend(30.0);
        p.record_spend(20.0);
        assert!((p.spent_usd - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_project_remaining() {
        let mut p = Project::new("api", Some(100.0));
        p.record_spend(40.0);
        assert!((p.remaining_usd().unwrap() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_project_over_budget() {
        let mut p = Project::new("api", Some(10.0));
        p.record_spend(15.0);
        assert!(p.is_over_budget());
    }

    #[test]
    fn test_project_no_budget_not_over() {
        let mut p = Project::new("api", None);
        p.record_spend(1000.0);
        assert!(!p.is_over_budget());
    }

    #[test]
    fn test_team_total_spend() {
        let mut team = Team::new("eng", 500.0);
        team.record_spend("proj_a", 100.0);
        team.record_spend("proj_b", 200.0);
        assert!((team.total_spent_usd() - 300.0).abs() < 1e-9);
    }

    #[test]
    fn test_team_remaining() {
        let mut team = Team::new("eng", 500.0);
        team.record_spend("proj_a", 100.0);
        assert!((team.remaining_usd() - 400.0).abs() < 1e-9);
    }

    #[test]
    fn test_team_over_budget() {
        let mut team = Team::new("eng", 100.0);
        team.record_spend("proj_a", 150.0);
        assert!(team.is_over_budget());
    }

    #[test]
    fn test_org_total_spend() {
        let mut org = Organization::new("Acme", 1000.0);
        org.record_spend("team_a", "proj_1", 200.0);
        org.record_spend("team_b", "proj_2", 300.0);
        assert!((org.total_spent_usd() - 500.0).abs() < 1e-9);
    }

    #[test]
    fn test_org_summary_sorted_by_spend() {
        let mut org = Organization::new("Acme", 1000.0);
        org.add_team(Team::new("small", 100.0));
        org.add_team(Team::new("big", 900.0));
        org.record_spend("small", "p", 10.0);
        org.record_spend("big", "p", 500.0);
        let summary = org.summary();
        assert_eq!(summary.teams[0].name, "big");
        assert_eq!(summary.teams[1].name, "small");
    }

    #[test]
    fn test_org_tree_lines_non_empty() {
        let mut org = Organization::new("Corp", 100.0);
        org.record_spend("eng", "api", 25.0);
        let lines = org.tree_lines();
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Corp"));
    }

    #[test]
    fn test_auto_create_team_on_spend() {
        let mut org = Organization::new("Corp", 100.0);
        org.record_spend("new_team", "proj", 5.0);
        assert!(org.teams.contains_key("new_team"));
    }

    #[test]
    fn test_pct_consumed() {
        let mut org = Organization::new("Corp", 200.0);
        org.record_spend("t", "p", 100.0);
        assert!((org.pct_consumed() - 0.5).abs() < 1e-9);
    }
}
