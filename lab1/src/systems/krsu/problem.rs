use crate::core::common::{
    CommonProblemSummary, CommonTestGroup, DataStreamSource, ProblemResourceLimits,
};

/// Evaluation run type for KRSU system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KrsuRunEvaluationType {
    /// Stop testing on the first failed test (0)
    StopOnFirstFailure,
    /// Run all tests unconditionally (1)
    RunAllTests,
    /// Evaluate by test groups (2)
    EvaluateByGroups,
    /// Custom / other run type
    Other(i32),
}

impl KrsuRunEvaluationType {
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::StopOnFirstFailure,
            1 => Self::RunAllTests,
            2 => Self::EvaluateByGroups,
            other => Self::Other(other),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::StopOnFirstFailure => "Stop at first failed test (0)",
            Self::RunAllTests => "Run all tests (1)",
            Self::EvaluateByGroups => "Evaluate by test groups (2)",
            Self::Other(_) => "Custom run type",
        }
    }
}

/// A test group definition in KRSU testinfo.xml.
#[derive(Debug, Clone, PartialEq)]
pub struct KrsuTestGroup {
    pub group_identifier: String,
    pub score_points: f64,
    pub prerequisite_group_identifiers: Vec<String>,
}

/// A single test case definition in KRSU testinfo.xml.
#[derive(Debug, Clone, PartialEq)]
pub struct KrsuTestCase {
    pub input_file_path: String,
    pub output_file_path: String,
    pub group_identifier: Option<String>,
    pub score_points: Option<f64>,
}

/// Complete representation of a KRSU problem package.
#[derive(Debug, Clone, PartialEq)]
pub struct KrsuProblem {
    pub checker_executable: Option<String>,
    pub interactor_executable: Option<String>,
    pub problem_statement_file_path: Option<String>,
    pub memory_limit_bytes: Option<u64>,
    pub time_limit_milliseconds: Option<u64>,
    pub package_test_version: Option<u32>,
    pub run_evaluation_type: Option<KrsuRunEvaluationType>,
    pub test_groups: Vec<KrsuTestGroup>,
    pub test_cases: Vec<KrsuTestCase>,
    pub common_summary: CommonProblemSummary,
}

impl KrsuProblem {
    pub fn build_common_summary(&self) -> CommonProblemSummary {
        let total_test_cases_count = self.test_cases.len();
        let total_test_groups_count = self.test_groups.len();

        let test_groups = self
            .test_groups
            .iter()
            .map(|g| {
                let test_count = self
                    .test_cases
                    .iter()
                    .filter(|t| t.group_identifier.as_deref() == Some(&g.group_identifier))
                    .count();

                CommonTestGroup {
                    group_identifier: g.group_identifier.clone(),
                    group_description: None,
                    total_score_points: Some(g.score_points),
                    single_test_points: None,
                    prerequisite_group_identifiers: g.prerequisite_group_identifiers.clone(),
                    included_test_count: test_count,
                }
            })
            .collect();

        CommonProblemSummary {
            problem_title: self.problem_statement_file_path.clone(),
            problem_identifier: None,
            resource_limits: ProblemResourceLimits {
                time_limit_milliseconds: self.time_limit_milliseconds,
                memory_limit_bytes: self.memory_limit_bytes,
                write_limit_bytes: None,
            },
            input_stream: DataStreamSource::StandardInputOutput,
            output_stream: DataStreamSource::StandardInputOutput,
            checker_description: self.checker_executable.clone(),
            interactor_description: self.interactor_executable.clone(),
            total_test_cases_count,
            total_test_groups_count,
            sample_tests_count: 0,
            test_groups,
            sample_tests: Vec::new(),
        }
    }
}