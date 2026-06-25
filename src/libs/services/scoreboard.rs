use super::ServiceError;
use crate::libs::repos::{ChallengeRepo, SubmissionRepo, TeamRepo};
use crate::libs::types::challenges::{Challenge, ScoringMode};
use crate::libs::types::scoreboard::{
    CtfTimeScoreboardExport, CtfTimeStandingsEntry, CtfTimeTaskStats, ScoreboardEntry,
};
use crate::libs::types::solves::Submission;
use crate::libs::types::teams::TeamId;
use std::collections::HashMap;

pub struct ScoreboardService<T, C, S>
where
    T: TeamRepo,
    C: ChallengeRepo,
    S: SubmissionRepo,
{
    pub team_repo: T,
    pub challenge_repo: C,
    pub submission_repo: S,
    pub sort_by_accuracy: bool,
}

impl<T, C, S> ScoreboardService<T, C, S>
where
    T: TeamRepo,
    C: ChallengeRepo,
    S: SubmissionRepo,
{
    pub async fn get_scoreboard(&self) -> Result<Vec<ScoreboardEntry>, ServiceError> {
        let teams = self.team_repo.find_all().await?;
        let submissions = self.submission_repo.find_all().await?;
        let challenges = self.challenge_repo.find_all().await?;
        let challenge_map: HashMap<String, &Challenge> =
            challenges.iter().map(|c| (c.id.clone(), c)).collect();
        let mut solve_counts = HashMap::new();
        for sub in &submissions {
            if sub.is_correct {
                *solve_counts.entry(sub.challenge_id.clone()).or_insert(0) += 1;
            }
        }
        let mut entries = Vec::new();
        for team in teams {
            let team_subs: Vec<&Submission> = submissions
                .iter()
                .filter(|s| s.team_id.as_ref() == Some(&team.id))
                .collect();
            let mut points = 0;
            let mut last_solve_time = None;
            let mut solved_ids = Vec::new();
            for sub in team_subs {
                if sub.is_correct {
                    if let Some(challenge) = challenge_map.get(&sub.challenge_id) {
                        let challenge_points = match challenge.points.mode {
                            ScoringMode::PointValue => {
                                challenge.points.equation.parse::<u32>().unwrap_or(100)
                            }
                            ScoringMode::PointAttribution => sub.points,
                        };
                        points += challenge_points;
                        solved_ids.push(sub.challenge_id.clone());

                        last_solve_time = match last_solve_time {
                            None => Some(sub.submitted_at),
                            Some(t) => Some(t.max(sub.submitted_at)),
                        };
                    }
                }
            }
            entries.push(ScoreboardEntry {
                team_id: team.id,
                team_name: team.name.0,
                points,
                last_solve_time,
                solves: solved_ids,
                rank: 0,
            });
        }
        if self.sort_by_accuracy {
            let get_accuracy = |team_id: &TeamId| -> f64 {
                let subs: Vec<&Submission> = submissions
                    .iter()
                    .filter(|s| s.team_id.as_ref() == Some(team_id))
                    .collect();
                if subs.is_empty() {
                    1.0
                } else {
                    (subs.iter().filter(|s| s.is_correct).count() as f64) / (subs.len() as f64)
                }
            };
            entries.sort_by(|a, b| {
                b.points.cmp(&a.points).then_with(|| {
                    let acc_a = get_accuracy(&a.team_id);
                    let acc_b = get_accuracy(&b.team_id);
                    acc_b
                        .partial_cmp(&acc_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });
        } else {
            entries.sort_by(|a, b| {
                b.points
                    .cmp(&a.points)
                    .then_with(|| match (a.last_solve_time, b.last_solve_time) {
                        (Some(t1), Some(t2)) => t1.cmp(&t2),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    })
            });
        }
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.rank = (i + 1) as u32;
        }
        Ok(entries)
    }

    pub async fn export_ctftime(&self) -> Result<CtfTimeScoreboardExport, ServiceError> {
        let standings = self.get_scoreboard().await?;
        let submissions = self.submission_repo.find_all().await?;
        let challenges = self.challenge_repo.find_all().await?;
        let challenge_map: HashMap<String, &Challenge> =
            challenges.iter().map(|c| (c.id.clone(), c)).collect();
        let tasks: Vec<String> = challenges.iter().map(|c| c.title.0.clone()).collect();
        let mut ctftime_standings = Vec::new();
        for entry in standings {
            let mut task_stats = HashMap::new();
            let team_solves: Vec<&Submission> = submissions
                .iter()
                .filter(|s| s.team_id.as_ref() == Some(&entry.team_id) && s.is_correct)
                .collect();
            for solve in team_solves {
                if let Some(challenge) = challenge_map.get(&solve.challenge_id) {
                    task_stats.insert(
                        challenge.title.0.clone(),
                        CtfTimeTaskStats {
                            points: solve.points,
                            time: solve.submitted_at,
                        },
                    );
                }
            }
            ctftime_standings.push(CtfTimeStandingsEntry {
                pos: Some(entry.rank),
                team: entry.team_name,
                score: entry.points as f64,
                task_stats,
            });
        }
        Ok(CtfTimeScoreboardExport {
            tasks,
            standings: ctftime_standings,
        })
    }
}
