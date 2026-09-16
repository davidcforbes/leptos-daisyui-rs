//! All copy the `Helpdesk` composite renders, in one struct built by literal.

use super::model::{TicketKind, TicketPriority};

/// Every string the composite, its drawer and its dialog render. `Default`
/// is English; a host overrides individual fields by struct-update syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpdeskTexts {
    /// Page/composite title.
    pub title: String,
    /// The header's "file a new one" button.
    pub new_request: String,
    /// The header's manual-refresh button.
    pub refresh: String,
    /// Accessible name of the bucket group.
    pub buckets_label: String,
    /// Label for the Open bucket.
    pub bucket_open: String,
    /// Label for the In Progress bucket.
    pub bucket_in_progress: String,
    /// Label for the Done bucket.
    pub bucket_done: String,
    /// Label for clearing the bucket filter.
    pub bucket_all: String,
    /// Placeholder for the search box.
    pub search_placeholder: String,
    /// Label for the priority filter.
    pub filter_priority: String,
    /// Label for the assignee filter (support only).
    pub filter_assignee: String,
    /// The priority filter's "no filter" option.
    pub any_priority: String,
    /// The assignee filter's "no filter" option.
    pub any_assignee: String,
    /// Label for the "mine only" toggle (support only).
    pub mine_only: String,
    /// Label for resetting every filter.
    pub reset: String,
    /// Key column header.
    pub col_key: String,
    /// Summary column header.
    pub col_summary: String,
    /// Priority column header.
    pub col_priority: String,
    /// Status column header.
    pub col_status: String,
    /// Assignee column header.
    pub col_assignee: String,
    /// Age column header.
    pub col_age: String,
    /// Accessible name of one column-filter control in the table's filter
    /// row; `{column}` is replaced by the column header.
    pub filter_label: String,
    /// Placeholder inside a column text filter in the filter row.
    pub filter_placeholder: String,
    /// The reset option of a column's option-list filter.
    pub filter_all: String,
    /// Shown in place of an unset assignee or priority.
    pub unassigned: String,
    /// Display name for `TicketKind::Bug`.
    pub kind_bug: String,
    /// Display name for `TicketKind::Request`.
    pub kind_request: String,
    /// Display name for `TicketPriority::Highest`.
    pub priority_highest: String,
    /// Display name for `TicketPriority::High`.
    pub priority_high: String,
    /// Display name for `TicketPriority::Medium`.
    pub priority_medium: String,
    /// Display name for `TicketPriority::Low`.
    pub priority_low: String,
    /// Display name for `TicketPriority::Lowest`.
    pub priority_lowest: String,
    /// Display name for `TicketPriority::Unset`.
    pub priority_unset: String,
    /// Accessible name of the detail drawer.
    pub drawer_label: String,
    /// Quick action linking to the Jira issue.
    pub open_in_jira: String,
    /// Meta-row label for the requester.
    pub requester: String,
    /// Meta-row label for the created timestamp.
    pub created: String,
    /// Meta-row label for the updated timestamp.
    pub updated: String,
    /// Description section heading.
    pub description: String,
    /// Attachments section heading.
    pub attachments: String,
    /// Comments section heading.
    pub comments: String,
    /// Shown when a ticket has no comments.
    pub no_comments: String,
    /// Placeholder for the add-comment box.
    pub comment_placeholder: String,
    /// The add-comment submit button.
    pub add_comment: String,
    /// Label for the status select (support only).
    pub status: String,
    /// Label for the assignee select (support only).
    pub assignee: String,
    /// Label for the priority select (support only).
    pub priority: String,
    /// The drawer's close button.
    pub close: String,
    /// New Request dialog title.
    pub dialog_title: String,
    /// Label for the kind picker.
    pub kind: String,
    /// Label for the summary field.
    pub summary: String,
    /// Error shown when the summary is blank.
    pub summary_required: String,
    /// Error shown when the summary is too long; `{max}` substituted.
    pub summary_too_long: String,
    /// Placeholder for the description field, non-Bug kinds.
    pub description_placeholder: String,
    /// Placeholder for the description field, Bug kind.
    pub bug_placeholder: String,
    /// The read-only context line; `{route}`/`{build}` substituted.
    pub context_line: String,
    /// The dialog's cancel button.
    pub cancel: String,
    /// The dialog's submit button.
    pub submit: String,
    /// Toast shown after filing; `{key}` substituted.
    pub filed: String,
    /// Shown after a rate-limited submit; `{seconds}` substituted.
    pub rate_limited: String,
    /// Heading shown when the helpdesk is not configured.
    pub not_configured_title: String,
    /// Prefix for a submit failure message.
    pub submit_failed: String,
    /// Prefix for a failed triage action.
    pub action_failed: String,
    /// Announced after a successful triage action.
    pub saved: String,
}

impl Default for HelpdeskTexts {
    fn default() -> Self {
        Self {
            title: "Helpdesk".into(),
            new_request: "New request".into(),
            refresh: "Refresh".into(),
            buckets_label: "Ticket status".into(),
            bucket_open: "Open".into(),
            bucket_in_progress: "In progress".into(),
            bucket_done: "Done".into(),
            bucket_all: "All".into(),
            search_placeholder: "Search key or summary".into(),
            filter_priority: "Priority".into(),
            filter_assignee: "Assignee".into(),
            any_priority: "Any priority".into(),
            any_assignee: "Anyone".into(),
            mine_only: "Mine only".into(),
            reset: "Reset".into(),
            col_key: "Key".into(),
            col_summary: "Summary".into(),
            col_priority: "Priority".into(),
            col_status: "Status".into(),
            col_assignee: "Assignee".into(),
            col_age: "Age".into(),
            filter_label: "Filter {column}".into(),
            filter_placeholder: "Filter…".into(),
            filter_all: "All".into(),
            unassigned: "—".into(),
            kind_bug: "Bug".into(),
            kind_request: "Request".into(),
            priority_highest: "Highest".into(),
            priority_high: "High".into(),
            priority_medium: "Medium".into(),
            priority_low: "Low".into(),
            priority_lowest: "Lowest".into(),
            priority_unset: "—".into(),
            drawer_label: "Ticket detail".into(),
            open_in_jira: "Open in Jira".into(),
            requester: "Requester".into(),
            created: "Created".into(),
            updated: "Updated".into(),
            description: "Description".into(),
            attachments: "Attachments".into(),
            comments: "Comments".into(),
            no_comments: "No comments yet".into(),
            comment_placeholder: "Add a comment".into(),
            add_comment: "Comment".into(),
            status: "Status".into(),
            assignee: "Assignee".into(),
            priority: "Priority".into(),
            close: "Close".into(),
            dialog_title: "New request".into(),
            kind: "What is this?".into(),
            summary: "Summary".into(),
            summary_required: "A summary is required".into(),
            summary_too_long: "Keep the summary under {max} characters".into(),
            description_placeholder: "What would you like?".into(),
            bug_placeholder: "What I expected / What happened".into(),
            context_line: "Page {route} · Build {build}".into(),
            cancel: "Cancel".into(),
            submit: "Submit".into(),
            filed: "Filed {key}".into(),
            rate_limited: "Too many requests. Try again in {seconds} seconds.".into(),
            not_configured_title: "Helpdesk is not available".into(),
            submit_failed: "Could not file the request".into(),
            action_failed: "Change not saved".into(),
            saved: "Saved".into(),
        }
    }
}

impl HelpdeskTexts {
    /// The display name for a ticket kind.
    pub fn kind_name(&self, kind: &TicketKind) -> String {
        match kind {
            TicketKind::Bug => self.kind_bug.clone(),
            TicketKind::Request => self.kind_request.clone(),
            TicketKind::Other(s) => s.clone(),
        }
    }

    /// The display name for a priority.
    pub fn priority_name(&self, p: TicketPriority) -> String {
        match p {
            TicketPriority::Highest => self.priority_highest.clone(),
            TicketPriority::High => self.priority_high.clone(),
            TicketPriority::Medium => self.priority_medium.clone(),
            TicketPriority::Low => self.priority_low.clone(),
            TicketPriority::Lowest => self.priority_lowest.clone(),
            TicketPriority::Unset => self.priority_unset.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::helpdesk::model::{TicketKind, TicketPriority};

    #[test]
    fn names_resolve() {
        let t = HelpdeskTexts::default();
        assert_eq!(t.kind_name(&TicketKind::Bug), "Bug");
        assert_eq!(t.kind_name(&TicketKind::Other("Story".into())), "Story");
        assert_eq!(t.priority_name(TicketPriority::Unset), "—");
        assert!(t.rate_limited.contains("{seconds}"));
    }
}
