use crate::core::common::{
    CommonProblemSummary, CommonTestCase, CommonTestGroup, DataStreamSource, ProblemResourceLimits,
};

/// Development environment / compiler code information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsDevelopmentEnvironment {
    pub code: Option<String>,
    pub human_name: Option<String>,
}

impl CatsDevelopmentEnvironment {
    pub fn from_code(code: Option<&str>) -> Self {
        let code_str = code.map(|s| s.to_string());
        let human_name = code.map(|c| match c {
            "1" => "None / Header only (h;inc)",
            "2" => "Answers in zip (zip)",
            "101" => "Cross-platform C/C++ (cpp;c)",
            "102" => "GNU C++ (cc;cxx)",
            "103" => "MS Visual C++ (cpp;c)",
            "104" => "Borland C++ 3.1 (cpp;c)",
            "105" => "GNU C (c)",
            "201" => "Borland Pascal 7.0 (pas)",
            "202" => "Free Pascal (pp)",
            "203" => "Borland Delphi (dpr)",
            "301" => "Quick Basic (qb)",
            "302" => "Free Basic (bas)",
            "401" => "Java (java)",
            "402" => "C# (cs)",
            "501" => "Perl (pl)",
            "502" => "Python (py)",
            "503" => "Haskell (hs)",
            "504" => "Ruby (rb)",
            other => other,
        }.to_string());

        Self {
            code: code_str,
            human_name,
        }
    }
}

/// Checker style format for parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatsCheckerStyle {
    Legacy,
    Testlib,
    Partial,
    Multiple,
    Other(String),
}

impl CatsCheckerStyle {
    pub fn from_style(style: Option<&str>) -> Self {
        match style {
            Some("legacy") => Self::Legacy,
            Some("testlib") => Self::Testlib,
            Some("partial") => Self::Partial,
            Some("multiple") => Self::Multiple,
            Some(other) => Self::Other(other.to_string()),
            None => Self::Legacy,
        }
    }
}

/// Picture attachment description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsPicture {
    pub name: String,
    pub source_path: String,
}

/// Attachment file description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsAttachment {
    pub name: String,
    pub source_path: String,
}

/// Reference solution in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsSolution {
    pub solution_name: String,
    pub source_file_path: String,
    pub development_environment: CatsDevelopmentEnvironment,
    pub global_export_identifier: Option<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Custom checker program in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsChecker {
    pub checker_name: String,
    pub source_file_path: String,
    pub development_environment: CatsDevelopmentEnvironment,
    pub checker_parameter_style: CatsCheckerStyle,
    pub global_export_identifier: Option<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Test generator in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsGenerator {
    pub generator_name: String,
    pub source_file_path: String,
    pub output_file_pattern: Option<String>,
    pub development_environment: CatsDevelopmentEnvironment,
    pub global_export_identifier: Option<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Test validator in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsValidator {
    pub validator_name: String,
    pub source_file_path: String,
    pub development_environment: CatsDevelopmentEnvironment,
    pub global_export_identifier: Option<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Interactor in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsInteractor {
    pub interactor_name: String,
    pub source_file_path: String,
    pub development_environment: CatsDevelopmentEnvironment,
    pub global_export_identifier: Option<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
}

/// Sample input and output demonstration in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsSample {
    pub rank_specification: String,
    pub input_file_path: Option<String>,
    pub input_inline_text: Option<String>,
    pub output_file_path: Option<String>,
    pub output_inline_text: Option<String>,
}

/// Test input definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsTestInputDefinition {
    pub source_file_path: Option<String>,
    pub generator_reference_name: Option<String>,
    pub generator_parameters: Option<String>,
    pub use_group_generation: bool,
    pub validator_reference_name: Option<String>,
    pub file_checksum_hash: Option<String>,
    pub inline_content: Option<String>,
}

/// Test output definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsTestOutputDefinition {
    pub source_file_path: Option<String>,
    pub solution_reference_name: Option<String>,
    pub inline_content: Option<String>,
}

/// Single or discrete test definition in CATS.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsTest {
    pub rank_specification: String,
    pub score_points: Option<f64>,
    pub input_definition: Option<CatsTestInputDefinition>,
    pub output_definition: Option<CatsTestOutputDefinition>,
}

/// Test range definition in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsTestRange {
    pub range_start_number: u32,
    pub range_end_number: u32,
    pub points_per_test: Option<f64>,
    pub input_definition: Option<CatsTestInputDefinition>,
    pub output_definition: Option<CatsTestOutputDefinition>,
}

/// Testset definition in CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsTestset {
    pub testset_name: String,
    pub test_rank_specification: String,
    pub total_score_points: Option<f64>,
    pub hide_details: bool,
    pub comment_description: Option<String>,
    pub depends_on_tests_specification: Option<String>,
}

/// Imported module or standard resource in CATS package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsImport {
    pub global_unique_identifier: String,
    pub imported_object_type: Option<String>,
    pub local_alias_name: Option<String>,
}

/// Problem statement text and links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatsProblemStatement {
    pub content_stml: String,
    pub attachment_identifier: Option<String>,
    pub external_url: Option<String>,
}

/// Full problem representation for CATS package.
#[derive(Debug, Clone, PartialEq)]
pub struct CatsProblem {
    pub cats_specification_version: String,
    pub problem_title: String,
    pub supported_natural_languages: Vec<String>,
    pub time_limit_seconds: Option<f64>,
    pub memory_limit_bytes: Option<u64>,
    pub write_limit_bytes: Option<u64>,
    pub author_name: Option<String>,
    pub input_file_name: Option<String>,
    pub output_file_name: Option<String>,
    pub author_difficulty_rating: Option<u32>,
    pub standard_checker_identifier: Option<String>,
    pub maximum_score_points: Option<f64>,
    pub save_input_prefix_limit: Option<String>,
    pub save_output_prefix_limit: Option<String>,
    pub save_answer_prefix_limit: Option<String>,
    pub keyword_codes: Vec<String>,
    pub problem_statement: Option<CatsProblemStatement>,
    pub problem_constraints_description: Option<String>,
    pub input_format_description: Option<String>,
    pub output_format_description: Option<String>,
    pub solution_explanation_description: Option<String>,
    pub json_metadata: Option<String>,
    pub pictures: Vec<CatsPicture>,
    pub attachments: Vec<CatsAttachment>,
    pub reference_solutions: Vec<CatsSolution>,
    pub custom_checkers: Vec<CatsChecker>,
    pub test_generators: Vec<CatsGenerator>,
    pub test_validators: Vec<CatsValidator>,
    pub interactor: Option<CatsInteractor>,
    pub run_execution_method: Option<String>,
    pub sample_demonstrations: Vec<CatsSample>,
    pub test_definitions: Vec<CatsTest>,
    pub test_ranges: Vec<CatsTestRange>,
    pub test_sets: Vec<CatsTestset>,
    pub imported_dependencies: Vec<CatsImport>,
    pub common_summary: CommonProblemSummary,
}

impl CatsProblem {
    pub fn build_common_summary(&self) -> CommonProblemSummary {
        let time_limit_milliseconds = self
            .time_limit_seconds
            .map(|secs| (secs * 1000.0).round() as u64);

        let input_stream = match self.input_file_name.as_deref() {
            Some("*STDIN") | None => DataStreamSource::StandardInputOutput,
            Some("*NONE") => DataStreamSource::None,
            Some(file) => DataStreamSource::SpecificFile(file.to_string()),
        };

        let output_stream = match self.output_file_name.as_deref() {
            Some("*STDOUT") | None => DataStreamSource::StandardInputOutput,
            Some("*NONE") => DataStreamSource::None,
            Some(file) => DataStreamSource::SpecificFile(file.to_string()),
        };

        let checker_description = self
            .standard_checker_identifier
            .clone()
            .or_else(|| {
                self.imported_dependencies
                    .iter()
                    .find(|i| i.imported_object_type.as_deref() == Some("checker"))
                    .map(|i| i.global_unique_identifier.clone())
            })
            .or_else(|| {
                self.custom_checkers
                    .first()
                    .map(|c| format!("{} ({})", c.checker_name, c.source_file_path))
            });

        let interactor_description = self
            .interactor
            .as_ref()
            .map(|i| format!("{} ({})", i.interactor_name, i.source_file_path));

        let mut total_test_cases_count = 0usize;
        for range in &self.test_ranges {
            if range.range_end_number >= range.range_start_number {
                total_test_cases_count +=
                    (range.range_end_number - range.range_start_number + 1) as usize;
            }
        }
        total_test_cases_count += self.test_definitions.len();

        let total_test_groups_count = self.test_sets.len();
        let sample_tests_count = self.sample_demonstrations.len();

        let test_groups = self
            .test_sets
            .iter()
            .map(|set| CommonTestGroup {
                group_identifier: set.testset_name.clone(),
                group_description: set.comment_description.clone(),
                total_score_points: set.total_score_points,
                single_test_points: None,
                prerequisite_group_identifiers: set
                    .depends_on_tests_specification
                    .clone()
                    .into_iter()
                    .collect(),
                included_test_count: 0,
            })
            .collect();

        let sample_tests = self
            .sample_demonstrations
            .iter()
            .enumerate()
            .map(|(idx, sample)| CommonTestCase {
                test_index_or_number: Some(idx + 1),
                input_file_or_description: sample
                    .input_file_path
                    .clone()
                    .or_else(|| sample.input_inline_text.clone())
                    .unwrap_or_default(),
                answer_file_or_description: sample
                    .output_file_path
                    .clone()
                    .or_else(|| sample.output_inline_text.clone()),
                assigned_group_identifier: None,
                score_points: None,
                is_sample_test: true,
            })
            .collect();

        CommonProblemSummary {
            problem_title: Some(self.problem_title.clone()),
            problem_identifier: None,
            resource_limits: ProblemResourceLimits {
                time_limit_milliseconds,
                memory_limit_bytes: self.memory_limit_bytes,
                write_limit_bytes: self.write_limit_bytes,
            },
            input_stream,
            output_stream,
            checker_description,
            interactor_description,
            total_test_cases_count,
            total_test_groups_count,
            sample_tests_count,
            test_groups,
            sample_tests,
        }
    }
}