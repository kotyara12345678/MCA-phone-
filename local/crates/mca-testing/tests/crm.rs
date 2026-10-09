use mca_core::domain::application::ApplicationState;
use mca_core::domain::qualification::Qualification;
use mca_core::error::ProviderError;
use mca_core::ids::ApplicationId;
use mca_core::ports::crm::CrmLead;
use mca_core::ports::crm::CrmProvider;
use mca_testing::RecordingCrmProvider;

#[tokio::test]
async fn crm_records_leads_and_emails() {
    let crm = RecordingCrmProvider::new();
    let lead = CrmLead {
        application_id: ApplicationId::new(),
        summary: "summary".to_string(),
        state: ApplicationState::default(),
        qualification: Qualification::Pending,
        manager_email_subject: "subject".to_string(),
        transcript_excerpt: "excerpt".to_string(),
    };
    let pushed = crm.push_lead(lead.clone()).await.unwrap();
    assert!(pushed.accepted);
    assert_eq!(crm.leads(), vec![lead]);

    crm.deliver_email("manager@mca.example", "subject", "body")
        .await
        .unwrap();
    assert_eq!(crm.emails().len(), 1);
    assert_eq!(crm.emails()[0].0, "manager@mca.example");
}

#[tokio::test]
async fn crm_accepted_after_one_failure() {
    let crm = RecordingCrmProvider::new();
    crm.fail_next_push(ProviderError::NotConfigured { provider: "crm" });
    let lead = CrmLead {
        application_id: ApplicationId::new(),
        summary: String::new(),
        state: ApplicationState::default(),
        qualification: Qualification::Pending,
        manager_email_subject: String::new(),
        transcript_excerpt: String::new(),
    };
    assert!(crm.push_lead(lead.clone()).await.is_err());
    assert_eq!(crm.push_count(), 0);
    assert!(crm.push_lead(lead).await.is_ok());
    assert_eq!(crm.push_count(), 1);
}
