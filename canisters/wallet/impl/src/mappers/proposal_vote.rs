use crate::models::ProposalVote;
use ic_canister_core::utils::timestamp_to_rfc3339;
use uuid::Uuid;
use wallet_api::ProposalVoteDTO;

impl From<ProposalVote> for ProposalVoteDTO {
    fn from(vote: ProposalVote) -> Self {
        Self {
            user_id: Uuid::from_bytes(vote.user_id).hyphenated().to_string(),
            decided_at: timestamp_to_rfc3339(&vote.decided_dt),
            status: vote.status.into(),
            status_reason: vote.status_reason,
        }
    }
}