
use std::fmt;

/// Representation of standard stream or file redirection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataStreamSource {
    StandardInputOutput,
    SpecificFile(String),
    None,
}

impl fmt::Display for DataStreamSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StandardInputOutput => write!(f, "Standard I/O"),
            Self::SpecificFile(file_name) => write!(f, "File: {}", file_name),
            Self::None => write!(f, "None"),
        }
    }
}

/// Normalized execution and resource limits for a problem.
#[derive(Debug, Clone, PartialEq)]
pub struct ProblemResourceLimits {
    pub time_limit_milliseconds: Option<u64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Normalized summary of a single test case.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonTestCase {
    pub test_index_or_number: Option<usize>,
    pub input_file_or_description: String,
    pub answer_file_or_description: Option<String>,
    pub assigned_group_identifier: Option<String>,
    pub score_points: Option<f64>,
    pub is_sample_test: bool,
}

/// Normalized summary of a test group.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonTestGroup {
    pub group_identifier: String,
    pub group_description: Option<String>,
    pub total_score_points: Option<f64>,
    pub single_test_points: Option<f64>,
    pub prerequisite_group_identifiers: Vec<String>,
    pub included_test_count: usize,
}

/// Common problem summary extracted across all competitive programming formats.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonProblemSummary {
    pub problem_title: Option<String>,
    pub problem_identifier: Option<String>,
    pub resource_limits: ProblemResourceLimits,
    pub input_stream: DataStreamSource,
    pub output_stream: DataStreamSource,
    pub checker_description: Option<String>,
    pub interactor_description: Option<String>,
    pub total_test_cases_count: usize,
    pub total_test_groups_count: usize,
    pub sample_tests_count: usize,
    pub test_groups: Vec<CommonTestGroup>,
    pub sample_tests: Vec<CommonTestCase>,
}
