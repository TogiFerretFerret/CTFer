use crate::libs::repos::{ChallengeRepo, SubmissionRepo};
use crate::libs::types::accounts::AccountId;
use crate::libs::types::challenges::ScoringMode;
use crate::libs::types::flags::FlagValidator;
use crate::libs::types::solves::{Submission, SubmissionId};
use crate::libs::types::teams::TeamId;
use super::ServiceError;

pub struct SolveService<C, S>
where
    C: ChallengeRepo,
    S: SubmissionRepo,
{
    pub challenge_repo: C,
    pub submission_repo: S,
}

impl<C, S> SolveService<C, S>
where
    C: ChallengeRepo,
    S: SubmissionRepo,
{
    pub async fn submit_flag(
        &self,
        challenge_id: &str,
        team_id: Option<TeamId>,
        account_id: AccountId,
        submitted_flag: &str,
    ) -> Result<Submission, ServiceError> {
        let challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or_else(|| ServiceError::InvalidRequest("ctf-challenge-not-found".to_string()))?;
        if let Some(ref t_id) = team_id {
            let subs = self.submission_repo.find_by_team(t_id).await?;
            if subs
                .iter()
                .any(|s| s.challenge_id == challenge_id && s.is_correct)
            {
                return Err(ServiceError::InvalidRequest(
                    "ctf-already-solved".to_string(),
                ));
            }
        }
        let is_correct = match &challenge.flag {
            FlagValidator::Static(flag) => flag.trim() == submitted_flag.trim(),
            FlagValidator::Regex(pattern) => {
                let re = regex::Regex::new(pattern)
                    .map_err(|_| ServiceError::InvalidRequest("admin-invalid-regex".to_string()))?;
                re.is_match(submitted_flag.trim())
            }
            FlagValidator::Instanced => {
                let active_flag: Option<String> = self
                    .challenge_repo
                    .find_active_flag(challenge_id, team_id.as_ref(), &account_id)
                    .await?;
                match active_flag {
                    Some(flag) => flag.trim() == submitted_flag.trim(),
                    None => false,
                }
            }
            FlagValidator::Script(_) => false,
        };

        let _total_solves = self
            .submission_repo
            .find_all()
            .await?
            .iter()
            .filter(|s| s.challenge_id == challenge_id && s.is_correct)
            .count() as u32;

        let points_awarded = if is_correct {
            match challenge.points.mode {
                ScoringMode::PointValue => challenge.points.equation.parse::<u32>().unwrap_or(100),
                ScoringMode::PointAttribution => {
                    challenge.points.equation.parse::<u32>().unwrap_or(100)
                }
            }
        } else {
            0
        };

        let submission = Submission {
            id: SubmissionId(uuid::Uuid::new_v4().to_string()),
            challenge_id: challenge_id.to_string(),
            team_id,
            account_id,
            points: points_awarded,
            provided_flag: submitted_flag.to_string(),
            is_correct,
            submitted_at: chrono::Utc::now().timestamp(),
        };

        self.submission_repo.save(submission.clone()).await?;

        if !is_correct {
            return Err(ServiceError::InvalidRequest(
                "ctf-incorrect-flag".to_string(),
            ));
        }

        Ok(submission)
    }
}
