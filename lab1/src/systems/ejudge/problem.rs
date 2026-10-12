use std::collections::HashMap;

use crate::core::common::{
    CommonProblemSummary, CommonTestGroup, DataStreamSource, ProblemResourceLimits,
};

/// Problem configuration loaded from `problem.cfg` in EJUDGE system.
#[derive(Debug, Clone, PartialEq)]
pub struct EjudgeProblemConfig {
    pub use_standard_input: Option<bool>,
    pub use_standard_output: Option<bool>,
    pub use_correct_answer_file: Option<bool>,
    pub enable_testlib_mode: Option<bool>,
    pub time_limit_seconds: Option<f64>,
    pub real_time_limit_seconds: Option<f64>,
    pub maximum_virtual_memory_bytes: Option<u64>,
    pub maximum_virtual_memory_raw: Option<String>,
    pub full_problem_name_localized: Option<String>,
    pub full_problem_name_english: Option<String>,
    pub internal_problem_identifier: Option<String>,
    pub test_file_pattern: Option<String>,
    pub correct_answer_file_pattern: Option<String>,
    pub checker_command: Option<String>,
    pub solution_command: Option<String>,
    pub external_system_identifier: Option<String>,
    pub revision_number: Option<String>,
    pub custom_properties: HashMap<String, String>,
}

impl Default for EjudgeProblemConfig {
    fn default() -> Self {
        Self {
            use_standard_input: None,
            use_standard_output: None,
            use_correct_answer_file: None,
            enable_testlib_mode: None,
            time_limit_seconds: None,
            real_time_limit_seconds: None,
            maximum_virtual_memory_bytes: None,
            maximum_virtual_memory_raw: None,
            full_problem_name_localized: None,
            full_problem_name_english: None,
            internal_problem_identifier: None,
            test_file_pattern: None,
            correct_answer_file_pattern: None,
            checker_command: None,
            solution_command: None,
            external_system_identifier: None,
            revision_number: None,
            custom_properties: HashMap::new(),
        }
    }
}

/// Global settings in EJUDGE `valuer.cfg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjudgeValuerGlobalSettings {
    pub report_statistics_to_judges: bool,
    pub report_statistics_to_users: bool,
}

/// Test group definition from EJUDGE `valuer.cfg`.
#[derive(Debug, Clone, PartialEq)]
pub struct EjudgeValuerGroup {
    pub group_identifier: u32,
    pub test_range_start: Option<u32>,
    pub test_range_end: Option<u32>,
    pub total_group_score: Option<f64>,
    pub single_test_score: Option<f64>,
    pub required_group_identifiers: Vec<u32>,
    pub is_offline_evaluation: bool,
    pub sets_marked: bool,
    pub is_skipped: bool,
    pub sets_marked_if_passed_group_identifiers: Vec<u32>,
    pub minimum_passed_tests_count: Option<u32>,
    pub skip_if_not_rejudge: bool,
    pub report_group_statistics_to_judges: bool,
    pub report_group_statistics_to_users: bool,
    pub user_status_on_pass: Option<String>,
    pub test_all_cases: bool,
    pub zero_score_if_only_tests: Vec<u32>,
}

/// Complete problem representation for EJUDGE packages.
#[derive(Debug, Clone, PartialEq)]
pub struct EjudgeProblem {
    pub problem_configuration: EjudgeProblemConfig,
    pub valuer_global_settings: Option<EjudgeValuerGlobalSettings>,
    pub valuer_test_groups: Vec<EjudgeValuerGroup>,
    pub total_discovered_test_files_count: usize,
    pub common_summary: CommonProblemSummary,
}

impl EjudgeProblem {
    pub fn build_common_summary(&self) -> CommonProblemSummary {
        let time_limit_milliseconds = self
            .problem_configuration
            .time_limit_seconds
            .map(|secs| (secs * 1000.0).round() as u64);

        let input_stream = match self.problem_configuration.use_standard_input {
            Some(true) | None => DataStreamSource::StandardInputOutput,
            Some(false) => DataStreamSource::None,
        };

        let output_stream = match self.problem_configuration.use_standard_output {
            Some(true) | None => DataStreamSource::StandardInputOutput,
            Some(false) => DataStreamSource::None,
        };

        let mut tests_count_from_groups = 0usize;
        for g in &self.valuer_test_groups {
            if let (Some(start), Some(end)) = (g.test_range_start, g.test_range_end) {
                if end >= start {
                    tests_count_from_groups += (end - start + 1) as usize;
                }
            }
        }

        let total_test_cases_count = if tests_count_from_groups > 0 {
            tests_count_from_groups
        } else {
            self.total_discovered_test_files_count
        };

        let test_groups = self
            .valuer_test_groups
            .iter()
            .map(|g| {
                let count = match (g.test_range_start, g.test_range_end) {
                    (Some(s), Some(e)) if e >= s => (e - s + 1) as usize,
                    _ => 0,
                };

                CommonTestGroup {
                    group_identifier: g.group_identifier.to_string(),
                    group_description: None,
                    total_score_points: g.total_group_score,
                    single_test_points: g.single_test_score,
                    prerequisite_group_identifiers: g
                        .required_group_identifiers
                        .iter()
                        .map(|id| id.to_string())
                        .collect(),
                    included_test_count: count,
                }
            })
            .collect();

        CommonProblemSummary {
            problem_title: self
                .problem_configuration
                .full_problem_name_localized
                .clone()
                .or_else(|| self.problem_configuration.full_problem_name_english.clone())
                .or_else(|| self.problem_configuration.internal_problem_identifier.clone()),
            problem_identifier: self.problem_configuration.internal_problem_identifier.clone(),
            resource_limits: ProblemResourceLimits {
                time_limit_milliseconds,
                memory_limit_bytes: self.problem_configuration.maximum_virtual_memory_bytes,
                write_limit_bytes: None,
            },
            input_stream,
            output_stream,
            checker_description: self.problem_configuration.checker_command.clone(),
            interactor_description: None,
            total_test_cases_count,
            total_test_groups_count: self.valuer_test_groups.len(),
            sample_tests_count: 0,
            test_groups,
            sample_tests: Vec::new(),
        }
    }
}