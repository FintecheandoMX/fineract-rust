//! COB unit tests.
//! Tests domain models, command types, and basic validation.
//! Tenant isolation security tests live in repository.rs tests module.

use super::*;
use fineract_command::CommandRequest;
use fineract_core::ids::*;

#[cfg(test)]
mod domain_tests {
    use super::*;

    #[test]
    fn job_creation_validates() {
        let result = Job::new(
            JobId::new(1),
            TenantId::new(1),
            "".to_string(),
            JobType::CloseOfBusiness,
            "0 0 * * *".to_string(),
        );
        assert!(matches!(result, Err(CobError::EmptyName)));

        let result = Job::new(
            JobId::new(1),
            TenantId::new(1),
            "".to_string(),
            JobType::CloseOfBusiness,
            "".to_string(),
        );
        assert!(matches!(result, Err(CobError::EmptyName)));

        let job = Job::new(
            JobId::new(1),
            TenantId::new(1),
            "Daily COB".to_string(),
            JobType::CloseOfBusiness,
            "0 0 * * *".to_string(),
        )
        .unwrap();
        assert_eq!(job.name, "Daily COB");
        assert_eq!(job.job_type, JobType::CloseOfBusiness);
        assert!(job.active);
        assert_eq!(job.max_retries, 3);
        assert_eq!(job.timeout_seconds, 3600);
    }

    #[test]
    fn execution_status_transitions() {
        let exec = JobExecution::new(JobExecutionId::new(1), JobId::new(1), TenantId::new(1));
        assert_eq!(exec.status, ExecutionStatus::Pending);
        assert_eq!(exec.retry_count, 0);
        assert!(exec.started_at.is_none());
        assert!(exec.completed_at.is_none());
        assert!(exec.error_message.is_none());
    }

    #[test]
    fn cob_run_lifecycle() {
        let run = CobRun::new(vec![TenantId::new(1), TenantId::new(2)]);
        assert_eq!(run.tenant_ids.len(), 2);
        assert_eq!(run.status, CobRunStatus::Running);
        assert!(run.completed_at.is_none());
    }

    #[test]
    fn job_type_round_trip() {
        let types = vec![
            JobType::CloseOfBusiness,
            JobType::InterestAccrual,
            JobType::ChargeApplication,
            JobType::DataArchival,
            JobType::Custom("my_job".to_string()),
        ];
        for jt in types {
            let s = format!("{:?}", &jt);
            let parsed = match &jt {
                JobType::CloseOfBusiness => JobType::CloseOfBusiness,
                JobType::InterestAccrual => JobType::InterestAccrual,
                JobType::ChargeApplication => JobType::ChargeApplication,
                JobType::DataArchival => JobType::DataArchival,
                JobType::Custom(ref name) => JobType::Custom(name.clone()),
            };
            assert_eq!(format!("{:?}", parsed), s);
            assert_eq!(format!("{:?}", &parsed), s);
        }
    }

    #[test]
    fn cob_error_tenant_isolation_check() {
        let err = CobError::TenantIsolationViolation;
        assert!(err.is_tenant_isolation_violation());

        let err = CobError::NotFound(JobId::new(1));
        assert!(!err.is_tenant_isolation_violation());
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateJobCommand::COMMAND_TYPE, "CreateJob");
        assert_eq!(UpdateJobCommand::COMMAND_TYPE, "UpdateJob");
        assert_eq!(EnableJobCommand::COMMAND_TYPE, "EnableJob");
        assert_eq!(DisableJobCommand::COMMAND_TYPE, "DisableJob");
        assert_ne!(
            CreateJobCommand::COMMAND_TYPE,
            UpdateJobCommand::COMMAND_TYPE
        );
        assert_ne!(
            CreateJobCommand::COMMAND_TYPE,
            EnableJobCommand::COMMAND_TYPE
        );
        assert_ne!(
            CreateJobCommand::COMMAND_TYPE,
            DisableJobCommand::COMMAND_TYPE
        );
    }
}
